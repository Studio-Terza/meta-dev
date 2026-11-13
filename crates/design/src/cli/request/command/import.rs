use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::Path;

use design_model::{
    Casting, Confidence, Entry, Explicitness, Facet, Node, NodeId, NotesLine, Ordinal, Prefix, Project, Provenance,
    Segment, Sense, SenseKind, SpanRelation, Specificity, Title, Vague,
};
use design_store::{
    Cited, Directed, Imported, ImportedNotes, Imports, NextNode, ProjectRoot, Quoted, QuotedSegments, ShownNode, Spoken,
};
use file_ports::{Files, Text};
use grammar_notes::{self as notes, Cite, Date, Ending, Marking, Notes, Quote, Reading, Unclosed};
use newtype::{Counter, Wrapping};
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
    ending: Ending,
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
    node: Node<Vague>,
    parent: NodeId,
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
struct Speech {
    cite: Cite,
    message: Uuid,
    author: Authorship,
    segments: Vec<Segment>,
}

#[derive(Debug)]
enum Anchor<'heard> {
    Whole(&'heard Segment),
    Narrowed(BTreeSet<&'heard Segment>),
    Ambiguous(Ambiguity),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Ambiguity {
    cite: Cite,
    segments: Count,
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
    ambiguities: Vec<Ambiguity>,
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

#[derive(Clone, Debug, PartialEq, Eq)]
enum Ruled {
    Remark,
    Anchored,
    Unanchored,
    Ambiguous(Vec<Ambiguity>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Settlement {
    Imported,
    Unanchored(Count),
    Unreconciled { lines: Count, nodes: Count },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
        let quoted: QuotedSegments =
            entries.iter().flat_map(notes::Entry::quotes).filter_map(Result::ok).cloned().collect();
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
        let imported = draft.drafted.into_iter().map(Drafted::imported).collect();
        self.store.write(Imports::new(draft.ending, imported))?;
        let nodes = Count::new(self.store.read(ImportedNotes::from(self.project.clone()))?.len());
        Ok(tally.reconciled(nodes, lines))
    }
}

impl Answers {
    fn drafted(&self, notes: &Notes, project: &Project) -> Draft {
        let start = Draft { next: self.next, section: self.root, drafted: Vec::new(), ending: notes.ending() };
        notes.lines().iter().fold(start, |draft, line| line.read_by(Taking { draft, heard: self, project }))
    }

    fn resolved(&self, entry: &notes::Entry, node: NodeId, voice: Voice) -> Findings {
        let citations: Vec<Sought<Speech>> = entry
            .cites()
            .into_iter()
            .map(|cite| Sought { reference: Reference::Cite(cite.clone()), found: self.cite(cite) })
            .collect();
        let authors: Vec<&Speech> = citations.iter().filter_map(|cited| cited.found.as_ref().ok()).collect();
        let cited: BTreeSet<Uuid> = authors.iter().map(|author| author.message.clone()).collect();
        let dates = entry.dates();
        let quotations: Vec<Sought<Vec<Segment>>> = entry
            .quotes()
            .into_iter()
            .map(|quote| {
                quote.map_or_else(Sought::unclosed, |closed| Sought {
                    reference: Reference::Quote(closed.clone()),
                    found: self.quote(closed, &cited, &dates),
                })
            })
            .collect();
        let quoted: BTreeSet<&Segment> =
            quotations.iter().filter_map(|quoted| quoted.found.as_ref().ok()).flatten().collect();
        let owned: Vec<&Speech> = authors.iter().copied().filter(|author| author.author == Authorship::Owner).collect();
        let anchors = Count::new(quoted.len().saturating_add(owned.len()));
        let resolved = voice.anchored(quoted, &owned);
        let placed: BTreeSet<&Segment> = resolved.iter().flat_map(Anchor::segments).collect();
        let ambiguities: Vec<Ambiguity> = resolved.into_iter().filter_map(Anchor::ambiguity).collect();
        let referenced: BTreeSet<Uuid> = cited.iter().cloned().chain(voice.referenced(&placed)).collect();
        Findings {
            directed: referenced
                .iter()
                .map(|message| Directed::Cited { node, message: message.clone() })
                .chain(placed.iter().flat_map(|segment| voice.directed((*segment).clone(), node)))
                .collect(),
            misses: citations
                .iter()
                .filter_map(Sought::missed)
                .chain(quotations.iter().filter_map(Sought::missed))
                .collect(),
            anchors,
            ambiguities,
        }
    }

    fn cite(&self, cite: &Cite) -> Result<Speech, Miss> {
        let spoken = self.cited.get(&Prefix::new(cite.clone().into_inner())).map_or_default(Vec::as_slice);
        let authors = spoken.iter().fold(BTreeMap::<Uuid, Speech>::new(), |mut authors, said| {
            let message: &Uuid = said.segment().as_ref();
            authors
                .entry(message.clone())
                .or_insert_with(|| Speech {
                    cite: cite.clone(),
                    message: message.clone(),
                    author: said.author(),
                    segments: Vec::new(),
                })
                .segments
                .push(said.segment().clone());
            authors
        });
        Miss::only(authors)
    }

    fn quote(&self, quote: &Quote, cited: &BTreeSet<Uuid>, dates: &[&Date]) -> Result<Vec<Segment>, Miss> {
        let found = self.quoted.get(quote).map_or_default(Vec::as_slice);
        let utterances = found.iter().fold(BTreeMap::<Occurrence, Utterance>::new(), |mut said, quoted| {
            let segment: &Segment = quoted.as_ref();
            let message: &Uuid = segment.as_ref();
            said.entry(Occurrence { message: message.clone(), ordinal: quoted.occurrence() })
                .or_insert_with(|| Utterance { day: quoted.day().clone(), segments: Vec::new() })
                .segments
                .push(segment.clone());
            said
        });
        let narrowed = Utterance::narrowed(utterances, cited);
        let same_day: BTreeSet<Uuid> = narrowed
            .iter()
            .filter(|(_occurrence, utterance)| dates.contains(&&utterance.day))
            .map(|(occurrence, _utterance)| occurrence.message.clone())
            .collect();
        Miss::only(Utterance::narrowed(narrowed, &same_day)).map(|utterance| utterance.segments)
    }
}

impl Speech {
    fn anchor(&self) -> Anchor<'_> {
        <&[Segment; 1]>::try_from(self.segments.as_slice()).map_or_else(
            |_several| {
                Anchor::Ambiguous(Ambiguity { cite: self.cite.clone(), segments: Count::new(self.segments.len()) })
            },
            |[whole]| Anchor::Whole(whole),
        )
    }

    fn anchors<'heard>(owned: &[&'heard Speech]) -> Vec<Anchor<'heard>> {
        owned.iter().map(|speech| speech.anchor()).collect()
    }
}

impl<'heard> Anchor<'heard> {
    fn segments(&self) -> Vec<&'heard Segment> {
        match self {
            Anchor::Whole(segment) => vec![*segment],
            Anchor::Narrowed(segments) => segments.iter().copied().collect(),
            Anchor::Ambiguous(_ambiguity) => Vec::new(),
        }
    }

    fn ambiguity(self) -> Option<Ambiguity> {
        match self {
            Anchor::Ambiguous(ambiguity) => Some(ambiguity),
            Anchor::Whole(_) | Anchor::Narrowed(_) => None,
        }
    }
}

impl Ambiguity {
    fn line(&self, line: NotesLine) -> Noted {
        Noted::Ambiguous { line, cite: self.cite.clone(), segments: self.segments }
    }
}

impl Utterance {
    fn narrowed(said: BTreeMap<Occurrence, Utterance>, chosen: &BTreeSet<Uuid>) -> BTreeMap<Occurrence, Utterance> {
        let (kept, rest): (BTreeMap<Occurrence, Utterance>, BTreeMap<Occurrence, Utterance>) =
            said.into_iter().partition(|(occurrence, _said)| chosen.contains(&occurrence.message));
        [kept, rest].into_iter().find(|share| share.first_key_value().is_some()).unwrap_or_default()
    }
}

impl Miss {
    fn only<Key, Value>(found: BTreeMap<Key, Value>) -> Result<Value, Miss> {
        <[Value; 1]>::try_from(found.into_values().collect::<Vec<Value>>())
            .map(|[only]| only)
            .map_err(|several| several.first().map_or(Miss::Unheard, |_several| Miss::Ambiguous))
    }
}

impl<Found> Sought<Found> {
    fn unclosed(unclosed: &Unclosed) -> Sought<Found> {
        Sought { reference: Reference::Quote(unclosed.clone().into_inner()), found: Err(Miss::Unclosed) }
    }

    fn missed(&self) -> Option<Missed> {
        self.found.as_ref().err().map(|miss| Missed { reference: self.reference.clone(), miss: *miss })
    }
}

impl Reading for Entries {
    type Output = Option<notes::Entry>;

    fn layout(self, _line: &notes::Line) -> Option<notes::Entry> {
        None
    }

    fn entry(self, _line: &notes::Line, entry: &notes::Entry, _marking: Marking) -> Option<notes::Entry> {
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
        let Findings { directed, misses, anchors, ambiguities } = self.heard.resolved(entry, node.identity(), voice);
        let drafted = Drafted {
            line: NotesLine::new(line.number()),
            node,
            parent: parent.node,
            directed,
            misses,
            ruled: voice.judged(anchors, ambiguities),
        };
        self.draft.after(drafted, parent)
    }
}

impl Reading for Taking<'_> {
    type Output = Draft;

    fn layout(self, _line: &notes::Line) -> Draft {
        self.draft
    }

    fn entry(self, line: &notes::Line, entry: &notes::Entry, marking: Marking) -> Draft {
        match marking {
            Marking::Heading => self.headed(line, entry),
            Marking::Ruling => self.entered(line, entry, Voice::Ruling),
            Marking::Lead | Marking::Remark => self.entered(line, entry, Voice::Remark),
        }
    }
}

impl Taking<'_> {
    fn headed(self, line: &notes::Line, entry: &notes::Entry) -> Draft {
        let Taking { draft, heard, project } = self;
        let rooted = Taking { draft: Draft { section: heard.root, ..draft }, heard, project };
        let node = rooted.node(heard.root, line);
        let section = Parent { node: node.identity(), casting: node.casting() };
        Draft { section, ..rooted.entered(line, entry, Voice::Remark) }
    }
}

impl Draft {
    fn after(self, drafted: Drafted, section: Parent) -> Draft {
        Draft {
            next: NodeId::new(self.next.into_inner().saturating_add(1)),
            section,
            drafted: self.drafted.into_iter().chain([drafted]).collect(),
            ending: self.ending,
        }
    }

    fn landing(&self) -> Landing {
        self.drafted.iter().map(|drafted| drafted.ruled.landing()).max().unwrap_or(Landing::Written)
    }

    fn refused(self) -> Report {
        let rulings =
            Count::new(self.drafted.iter().filter(|drafted| drafted.ruled.landing() == Landing::Refused).count());
        Report {
            lines: self.drafted.iter().flat_map(|drafted| drafted.lines(Landing::Refused)).collect(),
            settlement: Settlement::Unanchored(rulings),
        }
    }
}

impl Drafted {
    fn lines(&self, landing: Landing) -> Vec<Line> {
        let imported = landing.imported(self.line, self.node.identity());
        let unresolved = self.misses.iter().map(|missed| missed.line(self.line));
        let refusals = landing.refusals(self.line, &self.ruled);
        imported.into_iter().chain(unresolved).chain(refusals).map(Line::Noted).collect()
    }

    fn imported(self) -> Imported {
        Imported::new(Provenance::Import(self.line), self.node, self.parent, self.directed)
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
            rulings: Count::new(drafted.iter().filter(|draft| draft.ruled != Ruled::Remark).count()),
            anchored: Count::new(drafted.iter().filter(|draft| draft.ruled == Ruled::Anchored).count()),
        }
    }

    fn reconciled(self, nodes: Count, lines: Vec<Line>) -> Report {
        let Census { came, rulings, anchored } = self;
        let reconciled = Line::Noted(Noted::Reconciled(Reconciliation::new(came, nodes, rulings, anchored)));
        Report {
            lines: lines.into_iter().chain([reconciled]).collect(),
            settlement: Some(nodes)
                .filter(|made| *made == came)
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

    fn refusals(self, line: NotesLine, ruled: &Ruled) -> Vec<Noted> {
        match self {
            Landing::Written => Vec::new(),
            Landing::Refused => ruled.refusals(line),
        }
    }
}

impl Voice {
    fn directed(self, segment: Segment, node: NodeId) -> Vec<Directed> {
        match self {
            Voice::Ruling => vec![
                Directed::Placed { segment: segment.clone(), node, relation: SpanRelation::Decides },
                Directed::Sensed {
                    segment,
                    sense: Sense::new(
                        SenseKind::Decision,
                        Specificity::Specific,
                        Confidence::Firm,
                        Explicitness::Explicit,
                    ),
                },
            ],
            Voice::Remark => Vec::new(),
        }
    }

    fn referenced(self, placed: &BTreeSet<&Segment>) -> BTreeSet<Uuid> {
        match self {
            Voice::Ruling => BTreeSet::new(),
            Voice::Remark => placed
                .iter()
                .map(|segment| {
                    let message: &Uuid = (*segment).as_ref();
                    message.clone()
                })
                .collect(),
        }
    }

    fn anchored<'heard>(self, quoted: BTreeSet<&'heard Segment>, owned: &[&'heard Speech]) -> Vec<Anchor<'heard>> {
        match self {
            Voice::Ruling => Some(quoted)
                .filter(|segments| segments.first().is_some())
                .map_or_else(|| Speech::anchors(owned), |segments| vec![Anchor::Narrowed(segments)]),
            Voice::Remark => vec![Anchor::Narrowed(quoted)],
        }
    }

    fn judged(self, anchors: Count, ambiguities: Vec<Ambiguity>) -> Ruled {
        match self {
            Voice::Ruling => Ruled::anchoring(anchors, ambiguities),
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
    fn refusals(&self, line: NotesLine) -> Vec<Noted> {
        match self {
            Ruled::Unanchored => vec![Noted::Unanchored(line)],
            Ruled::Ambiguous(ambiguities) => ambiguities.iter().map(|ambiguity| ambiguity.line(line)).collect(),
            Ruled::Remark | Ruled::Anchored => Vec::new(),
        }
    }

    fn landing(&self) -> Landing {
        match self {
            Ruled::Unanchored | Ruled::Ambiguous(_) => Landing::Refused,
            Ruled::Remark | Ruled::Anchored => Landing::Written,
        }
    }

    fn anchoring(anchors: Count, ambiguities: Vec<Ambiguity>) -> Ruled {
        match ambiguities.first() {
            Some(_ambiguous) => Ruled::Ambiguous(ambiguities),
            None => Some(anchors)
                .filter(|count| *count != Count::zero())
                .map_or(Ruled::Unanchored, |_anchored| Ruled::Anchored),
        }
    }
}
