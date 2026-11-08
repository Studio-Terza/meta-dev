use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::Path;

use design_model::{
    Casting, Entry, Facet, Node, NodeId, NotesLine, Ordinal, Prefix, Project, Provenance, Segment, SpanRelation, Title,
    Vague,
};
use design_store::{
    Cited, Directed, Imported, ImportedNotes, Imports, NextNode, ProjectRoot, Quoted, QuotedSegments, ShownNode, Spoken,
};
use file_ports::{Files, Text};
use grammar::notes::{self, Cite, Date, Layout, Notes, Quote, Reading};
use newtype::{Counter, Newtype};
use store_ports::Store;
use transcript_ports::{Authorship, Uuid};

use super::Printed;
use crate::Outcome;
use crate::cli::failure::Failure;
use crate::cli::noted::{Miss, Noted, Reconciliation, Reference};
use crate::cli::output::{Count, Line};
use crate::cli::request::Ground;

#[derive(Debug)]
pub struct Import;

#[derive(Debug)]
struct Sheet<'store, S: Store> {
    store: &'store S,
    project: &'store Project,
}

#[derive(Clone, Copy, Debug)]
struct Parent {
    node: NodeId,
    casting: Casting,
}

#[derive(Debug)]
struct Answers {
    root: Parent,
    next: NodeId,
    cited: BTreeMap<Prefix, Vec<Spoken>>,
    quoted: BTreeMap<Quote, Vec<Quoted>>,
}

#[derive(Debug)]
struct Draft {
    next: NodeId,
    section: Parent,
    drafted: Vec<Drafted>,
}

#[derive(Debug)]
struct Taking<'heard> {
    draft: Draft,
    heard: &'heard Answers,
    project: &'heard Project,
}

#[derive(Clone, Copy, Debug)]
struct Entries;

#[derive(Debug)]
struct Drafted {
    line: NotesLine,
    node: NodeId,
    directed: Vec<Directed>,
    misses: Vec<Missed>,
    ruled: Ruled,
}

#[derive(Debug)]
struct Missed {
    reference: Reference,
    miss: Miss,
}

#[derive(Debug)]
struct Sought<Found> {
    reference: Reference,
    found: Result<Found, Miss>,
}

#[derive(Clone, Debug)]
struct Author {
    message: Uuid,
    author: Authorship,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Occurrence {
    message: Uuid,
    ordinal: Ordinal,
}

#[derive(Debug)]
struct Utterance {
    day: Date,
    segments: Vec<Segment>,
}

#[derive(Debug)]
struct Findings {
    directed: Vec<Directed>,
    misses: Vec<Missed>,
    anchors: Count,
}

#[derive(Clone, Copy, Debug)]
struct Census {
    came: Count,
    rulings: Count,
    anchored: Count,
}

#[derive(Debug)]
struct Report {
    lines: Vec<Line>,
    settlement: Settlement,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Voice {
    Ruling,
    Remark,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Ruled {
    Remark,
    Anchored,
    Unanchored,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Settlement {
    Imported,
    Unanchored(Count),
    Unreconciled { lines: Count, nodes: Count },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Landing {
    Written,
    Refused,
}

impl Import {
    pub fn written<F: Files, S: Store>(
        ground: &Ground<'_, F, S>,
        path: &Path,
        out: &mut impl Write,
    ) -> Result<Outcome, Failure> {
        let bytes = ground.read(path).map_err(Failure::Notes)?;
        let notes = Notes::read(Text::try_from(&bytes).map_err(Failure::Notes)?.as_ref());
        let project = ground.located().project().ok_or(Failure::Unconfigured)?;
        let sheet = Sheet { store: ground.store(), project };
        let lock = sheet.store.lock()?;
        let draft = sheet.heard(&notes)?.drafted(&notes, project);
        let report = draft.landing().landed(&sheet, draft)?;
        Printed::lines(out, &report.lines)?;
        lock.end()?;
        report.settlement.outcome()
    }
}

impl<S: Store> Sheet<'_, S> {
    fn heard(&self, notes: &Notes) -> Result<Answers, Failure> {
        let root = self.root()?;
        let earlier = self.store.read(ImportedNotes::from(self.project.clone()))?;
        let next = earlier.first().map_or_else(
            || Ok(self.store.read(NextNode)?),
            |_imported| Err(Failure::Reimported(self.project.clone())),
        )?;
        let entries: Vec<notes::Entry> = notes.lines().iter().filter_map(|line| line.read_by(Entries)).collect();
        let cited: Cited = entries
            .iter()
            .flat_map(notes::Entry::cites)
            .map(|cite: &Cite| Prefix::new(cite.clone().into_inner()))
            .collect();
        let quoted: QuotedSegments = entries.iter().flat_map(notes::Entry::quotes).cloned().collect();
        Ok(Answers { root, next, cited: self.store.read(cited)?, quoted: self.store.read(quoted)? })
    }

    fn root(&self) -> Result<Parent, Failure> {
        let root = self
            .store
            .read(ProjectRoot::from(self.project.clone()))?
            .ok_or_else(|| Failure::Unrooted(self.project.clone()))?
            .identity();
        let placed = self.store.read(ShownNode::from(root))?.ok_or(Failure::Unplaced(root))?;
        Ok(Parent { node: root, casting: placed.node().casting() })
    }

    fn written(&self, draft: Draft) -> Result<Report, Failure> {
        let tally = Census::counted(&draft.drafted);
        let lines: Vec<Line> = draft.drafted.iter().flat_map(|drafted| drafted.lines(Landing::Written)).collect();
        self.store.write(draft.drafted.into_iter().map(Drafted::imported).collect::<Imports>())?;
        let nodes = Count::new(self.store.read(ImportedNotes::from(self.project.clone()))?.len());
        Ok(tally.reconciled(nodes, lines))
    }
}

impl Answers {
    fn drafted(&self, notes: &Notes, project: &Project) -> Draft {
        let start = Draft { next: self.next, section: self.root, drafted: Vec::new() };
        notes.lines().iter().fold(start, |draft, line| line.read_by(Taking { draft, heard: self, project }))
    }

    fn resolved(&self, entry: &notes::Entry, node: NodeId, relation: SpanRelation) -> Findings {
        let citations: Vec<Sought<Author>> = entry
            .cites()
            .into_iter()
            .map(|cite| Sought { reference: Reference::Cite(cite.clone()), found: self.cite(cite) })
            .collect();
        let authors: Vec<&Author> = citations.iter().filter_map(|cited| cited.found.as_ref().ok()).collect();
        let cited: BTreeSet<Uuid> = authors.iter().map(|author| author.message.clone()).collect();
        let dates = entry.dates();
        let quotations: Vec<Sought<Vec<Segment>>> = entry
            .quotes()
            .into_iter()
            .map(|quote| Sought {
                reference: Reference::Quote(quote.clone()),
                found: self.quote(quote, &cited, &dates),
            })
            .collect();
        let placed: BTreeSet<&Segment> =
            quotations.iter().filter_map(|quoted| quoted.found.as_ref().ok()).flatten().collect();
        let owned = authors.iter().filter(|author| author.author == Authorship::Owner).count();
        Findings {
            directed: cited
                .iter()
                .map(|message| Directed::Cited { node, message: message.clone() })
                .chain(placed.iter().map(|segment| Directed::Placed { segment: (*segment).clone(), node, relation }))
                .collect(),
            misses: citations
                .iter()
                .filter_map(Sought::missed)
                .chain(quotations.iter().filter_map(Sought::missed))
                .collect(),
            anchors: Count::new(placed.len().saturating_add(owned)),
        }
    }

    fn cite(&self, cite: &Cite) -> Result<Author, Miss> {
        let spoken = self.cited.get(&Prefix::new(cite.clone().into_inner())).map(Vec::as_slice).unwrap_or_default();
        let authors: BTreeMap<Uuid, Author> = spoken
            .iter()
            .map(|spoken| {
                let message: &Uuid = spoken.segment().as_ref();
                (message.clone(), Author { message: message.clone(), author: spoken.author() })
            })
            .collect();
        Miss::only(authors)
    }

    fn quote(&self, quote: &Quote, cited: &BTreeSet<Uuid>, dates: &[&Date]) -> Result<Vec<Segment>, Miss> {
        let found = self.quoted.get(quote).map(Vec::as_slice).unwrap_or_default();
        let said = found.iter().fold(BTreeMap::<Occurrence, Utterance>::new(), |mut said, quoted| {
            let segment: &Segment = quoted.as_ref();
            let message: &Uuid = segment.as_ref();
            said.entry(Occurrence { message: message.clone(), ordinal: quoted.occurrence() })
                .or_insert_with(|| Utterance { day: quoted.day().clone(), segments: Vec::new() })
                .segments
                .push(segment.clone());
            said
        });
        let narrowed = Utterance::narrowed(said, cited);
        let same_day: BTreeSet<Uuid> = narrowed
            .iter()
            .filter(|(_occurrence, said)| dates.contains(&&said.day))
            .map(|(occurrence, _said)| occurrence.message.clone())
            .collect();
        Miss::only(Utterance::narrowed(narrowed, &same_day)).map(|said| said.segments)
    }
}

impl Utterance {
    fn narrowed(said: BTreeMap<Occurrence, Utterance>, chosen: &BTreeSet<Uuid>) -> BTreeMap<Occurrence, Utterance> {
        let (kept, rest): (BTreeMap<Occurrence, Utterance>, BTreeMap<Occurrence, Utterance>) =
            said.into_iter().partition(|(occurrence, _said)| chosen.contains(&occurrence.message));
        [kept, rest].into_iter().find(|said| said.first_key_value().is_some()).unwrap_or_default()
    }
}

impl Miss {
    fn only<Key, Value>(found: BTreeMap<Key, Value>) -> Result<Value, Miss> {
        <[Value; 1]>::try_from(found.into_values().collect::<Vec<Value>>())
            .map(|[only]| only)
            .map_err(|found| found.first().map_or(Miss::Unheard, |_several| Miss::Ambiguous))
    }
}

impl<Found> Sought<Found> {
    fn missed(&self) -> Option<Missed> {
        self.found.as_ref().err().map(|miss| Missed { reference: self.reference.clone(), miss: *miss })
    }
}

impl Reading for Entries {
    type Output = Option<notes::Entry>;

    fn layout(self, _line: &notes::Line, _layout: &Layout) -> Option<notes::Entry> {
        None
    }

    fn heading(self, _line: &notes::Line) -> Option<notes::Entry> {
        None
    }

    fn ruling(self, _line: &notes::Line, entry: &notes::Entry) -> Option<notes::Entry> {
        Some(entry.clone())
    }

    fn remark(self, _line: &notes::Line, entry: &notes::Entry) -> Option<notes::Entry> {
        Some(entry.clone())
    }
}

impl Taking<'_> {
    fn node(&self, parent: Parent, line: &notes::Line) -> Node<Vague> {
        let entry = Entry::new(self.project.clone(), Facet::Feature, Provenance::Import(NotesLine::new(line.number())));
        Node::proposed(self.draft.next, entry, parent.casting).titled(Title::from(line))
    }

    fn entered(self, line: &notes::Line, entry: &notes::Entry, voice: Voice) -> Draft {
        let parent = self.draft.section;
        let node = self.node(parent, line);
        let id = node.identity();
        let Findings { directed, misses, anchors } = self.heard.resolved(entry, id, voice.relation());
        let drafted = Drafted {
            line: NotesLine::new(line.number()),
            node: id,
            directed: [Directed::Proposed { node, parent: parent.node }].into_iter().chain(directed).collect(),
            misses,
            ruled: voice.judged(anchors),
        };
        self.draft.after(drafted, parent)
    }
}

impl Reading for Taking<'_> {
    type Output = Draft;

    fn layout(self, _line: &notes::Line, _layout: &Layout) -> Draft {
        self.draft
    }

    fn heading(self, line: &notes::Line) -> Draft {
        let root = self.heard.root;
        let node = self.node(root, line);
        let section = Parent { node: node.identity(), casting: node.casting() };
        let drafted = Drafted {
            line: NotesLine::new(line.number()),
            node: node.identity(),
            directed: vec![Directed::Proposed { node, parent: root.node }],
            misses: Vec::new(),
            ruled: Ruled::Remark,
        };
        self.draft.after(drafted, section)
    }

    fn ruling(self, line: &notes::Line, entry: &notes::Entry) -> Draft {
        self.entered(line, entry, Voice::Ruling)
    }

    fn remark(self, line: &notes::Line, entry: &notes::Entry) -> Draft {
        self.entered(line, entry, Voice::Remark)
    }
}

impl Draft {
    fn after(self, drafted: Drafted, section: Parent) -> Draft {
        Draft {
            next: NodeId::new(self.next.into_inner().saturating_add(1)),
            section,
            drafted: self.drafted.into_iter().chain([drafted]).collect(),
        }
    }

    fn landing(&self) -> Landing {
        let unanchored = self.drafted.iter().find(|drafted| drafted.ruled == Ruled::Unanchored);
        unanchored.map_or(Landing::Written, |_unanchored| Landing::Refused)
    }

    fn refused(self) -> Report {
        let rulings = Count::new(self.drafted.iter().filter(|drafted| drafted.ruled == Ruled::Unanchored).count());
        Report {
            lines: self.drafted.iter().flat_map(|drafted| drafted.lines(Landing::Refused)).collect(),
            settlement: Settlement::Unanchored(rulings),
        }
    }
}

impl Drafted {
    fn lines(&self, landing: Landing) -> Vec<Line> {
        let imported = landing.imported(self.line, self.node);
        let unresolved = self.misses.iter().map(|missed| missed.line(self.line));
        let unanchored = landing.unanchored(self.line, self.ruled);
        imported.into_iter().chain(unresolved).chain(unanchored).map(Line::Noted).collect()
    }

    fn imported(self) -> Imported {
        Imported::new(Provenance::Import(self.line), self.directed)
    }
}

impl Missed {
    fn line(&self, line: NotesLine) -> Noted {
        Noted::Unresolved { line, reference: self.reference.clone(), miss: self.miss }
    }
}

impl Census {
    fn counted(drafted: &[Drafted]) -> Census {
        Census {
            came: Count::new(drafted.len()),
            rulings: Count::new(drafted.iter().filter(|drafted| drafted.ruled != Ruled::Remark).count()),
            anchored: Count::new(drafted.iter().filter(|drafted| drafted.ruled == Ruled::Anchored).count()),
        }
    }

    fn reconciled(self, nodes: Count, lines: Vec<Line>) -> Report {
        let Census { came, rulings, anchored } = self;
        let reconciled = Line::Noted(Noted::Reconciled(Reconciliation::new(came, nodes, rulings, anchored)));
        Report {
            lines: lines.into_iter().chain([reconciled]).collect(),
            settlement: Some(nodes)
                .filter(|nodes| *nodes == came)
                .map_or(Settlement::Unreconciled { lines: came, nodes }, |_balanced| Settlement::Imported),
        }
    }
}

impl Landing {
    fn landed<S: Store>(self, sheet: &Sheet<'_, S>, draft: Draft) -> Result<Report, Failure> {
        match self {
            Landing::Written => sheet.written(draft),
            Landing::Refused => Ok(draft.refused()),
        }
    }

    fn imported(self, line: NotesLine, node: NodeId) -> Option<Noted> {
        match self {
            Landing::Written => Some(Noted::Imported { line, node }),
            Landing::Refused => None,
        }
    }

    fn unanchored(self, line: NotesLine, ruled: Ruled) -> Option<Noted> {
        match self {
            Landing::Written => None,
            Landing::Refused => ruled.unanchored(line),
        }
    }
}

impl Voice {
    fn relation(self) -> SpanRelation {
        match self {
            Voice::Ruling => SpanRelation::Decides,
            Voice::Remark => SpanRelation::Mentions,
        }
    }

    fn judged(self, anchors: Count) -> Ruled {
        match self {
            Voice::Ruling => Ruled::anchoring(anchors),
            Voice::Remark => Ruled::Remark,
        }
    }
}

impl Settlement {
    fn outcome(self) -> Result<Outcome, Failure> {
        match self {
            Settlement::Imported => Ok(Outcome::Imported),
            Settlement::Unanchored(rulings) => Err(Failure::Unanchored { rulings }),
            Settlement::Unreconciled { lines, nodes } => Err(Failure::Unreconciled { lines, nodes }),
        }
    }
}

impl Ruled {
    fn unanchored(self, line: NotesLine) -> Option<Noted> {
        match self {
            Ruled::Unanchored => Some(Noted::Unanchored(line)),
            Ruled::Remark | Ruled::Anchored => None,
        }
    }

    fn anchoring(anchors: Count) -> Ruled {
        Some(anchors).filter(|anchors| *anchors != Count::zero()).map_or(Ruled::Unanchored, |_anchored| Ruled::Anchored)
    }
}
