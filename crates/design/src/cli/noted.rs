use std::fmt;

use design_model::{NodeId, NotesLine, Prefix, Provenance};
use grammar_notes::{Cite, Quote};
use grammar_output::field::{Bare, Id, Integer, Quoted, Word};
use grammar_output::{Fields, Mapped, Parsed, Record, Unread};
use named_derive::Named;
use newtype::Wrapping;
use transcript_ports::Authorship;

use crate::cli::count::Count;
use crate::cli::output::Key;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Noted {
    Imported { line: NotesLine, node: NodeId },
    Unresolved { line: NotesLine, reference: Reference, miss: Miss },
    Unanchored(NotesLine),
    Ambiguous { line: NotesLine, cite: Cite, segments: Count },
    Reconciled(Reconciliation),
    Cites { message: Prefix, author: Authorship, entered: Provenance },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reference {
    Quote(Quote),
    Cite(Cite),
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Miss {
    Unheard,
    Ambiguous,
    Unclosed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Record)]
pub struct Reconciliation {
    #[past(Part::Lines)]
    #[field(Integer<_>, via = Count::new)]
    lines: Count,
    #[past(Part::Nodes)]
    #[field(Integer<_>, via = Count::new)]
    nodes: Count,
    #[past(Part::Rulings)]
    #[field(Integer<_>, via = Count::new)]
    rulings: Count,
    #[past(Part::Anchored)]
    #[field(Integer<_>, via = Count::new)]
    anchored: Count,
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
enum Part {
    Line,
    Quote,
    Cite,
    Lines,
    Nodes,
    Rulings,
    Anchored,
    Segments,
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
#[named(kebab)]
pub(crate) enum Note {
    Imported,
    Unresolved,
    Unanchored,
    AmbiguousAnchor,
    Reconciled,
    Cites,
}

#[derive(Debug, Record)]
struct Imported {
    #[past(Part::Line)]
    #[field(Integer<_>, via = NotesLine::new)]
    line: NotesLine,
    #[field(Id<_>)]
    node: NodeId,
}

#[derive(Debug, Record)]
struct Unresolved {
    #[past(Part::Line)]
    #[field(Integer<_>, via = NotesLine::new)]
    line: NotesLine,
    #[record]
    reference: Reference,
    #[field(Word<_>)]
    miss: Miss,
}

#[derive(Debug, Record)]
struct Unanchored {
    #[past(Part::Line)]
    #[field(Integer<_>, via = NotesLine::new)]
    line: NotesLine,
}

#[derive(Debug, Record)]
struct Ambiguous {
    #[past(Part::Line)]
    #[field(Integer<_>, via = NotesLine::new)]
    line: NotesLine,
    #[past(Part::Cite)]
    #[field(Bare<_>)]
    cite: Cite,
    #[past(Part::Segments)]
    #[field(Integer<_>, via = Count::new)]
    segments: Count,
}

#[derive(Debug, Record)]
struct Cites {
    #[field(Bare<_>)]
    message: Prefix,
    #[field(Word<_>)]
    author: Authorship,
    #[past(Key::Entered)]
    #[field(Word<_>)]
    entered: Provenance,
}

impl Note {
    pub(crate) fn noted(self, fields: Fields) -> Result<Parsed<Noted, Fields>, Unread> {
        match self {
            Note::Imported => fields.record::<Imported>().map(Parsed::converted),
            Note::Unresolved => fields.record::<Unresolved>().map(Parsed::converted),
            Note::Unanchored => fields.record::<Unanchored>().map(Parsed::converted),
            Note::AmbiguousAnchor => fields.record::<Ambiguous>().map(Parsed::converted),
            Note::Reconciled => fields.record::<Reconciliation>().map(Parsed::converted),
            Note::Cites => fields.record::<Cites>().map(Parsed::converted),
        }
    }
}

impl From<Imported> for Noted {
    fn from(read: Imported) -> Noted {
        Noted::Imported { line: read.line, node: read.node }
    }
}

impl From<Unresolved> for Noted {
    fn from(read: Unresolved) -> Noted {
        Noted::Unresolved { line: read.line, reference: read.reference, miss: read.miss }
    }
}

impl From<Unanchored> for Noted {
    fn from(read: Unanchored) -> Noted {
        Noted::Unanchored(read.line)
    }
}

impl From<Ambiguous> for Noted {
    fn from(read: Ambiguous) -> Noted {
        Noted::Ambiguous { line: read.line, cite: read.cite, segments: read.segments }
    }
}

impl From<Reconciliation> for Noted {
    fn from(read: Reconciliation) -> Noted {
        Noted::Reconciled(read)
    }
}

impl From<Cites> for Noted {
    fn from(read: Cites) -> Noted {
        Noted::Cites { message: read.message, author: read.author, entered: read.entered }
    }
}

impl Record for Reference {
    fn read(fields: Fields) -> Result<Parsed<Reference, Fields>, Unread> {
        fields.field::<Word<Part>>()?.concluded(Part::reference)
    }
}

impl Part {
    fn reference(self, fields: Fields) -> Result<Parsed<Reference, Fields>, Unread> {
        match self {
            Part::Quote => fields.field::<Quoted<_>>().mapped(Reference::Quote),
            Part::Cite => fields.field::<Bare<_>>().mapped(Reference::Cite),
            Part::Line | Part::Lines | Part::Nodes | Part::Rulings | Part::Anchored | Part::Segments => {
                Err(Unread::Unkeyed)
            }
        }
    }
}

impl Reconciliation {
    #[must_use]
    pub fn new(lines: Count, nodes: Count, rulings: Count, anchored: Count) -> Reconciliation {
        Reconciliation { lines, nodes, rulings, anchored }
    }
}

impl fmt::Display for Noted {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Noted::Imported { line, node } => {
                write!(formatter, "{} {} {line} n{}", Note::Imported, Part::Line, node.into_inner())
            }
            Noted::Unresolved { line, reference, miss } => {
                write!(formatter, "{} {} {line} {reference} {miss}", Note::Unresolved, Part::Line)
            }
            Noted::Unanchored(line) => write!(formatter, "{} {} {line}", Note::Unanchored, Part::Line),
            Noted::Ambiguous { line, cite, segments } => write!(
                formatter,
                "{} {} {line} {} {cite} {} {segments}",
                Note::AmbiguousAnchor,
                Part::Line,
                Part::Cite,
                Part::Segments
            ),
            Noted::Reconciled(reconciliation) => write!(formatter, "{} {reconciliation}", Note::Reconciled),
            Noted::Cites { message, author, entered } => {
                write!(formatter, "  {} {message} {author} {} {entered}", Note::Cites, Key::Entered)
            }
        }
    }
}

impl fmt::Display for Reference {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Reference::Quote(quote) => write!(formatter, "{} {}", Part::Quote, Fields::quoting(&quote.to_string())),
            Reference::Cite(cite) => write!(formatter, "{} {cite}", Part::Cite),
        }
    }
}

impl fmt::Display for Reconciliation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Reconciliation { lines, nodes, rulings, anchored } = self;
        write!(
            formatter,
            "{} {lines} {} {nodes} {} {rulings} {} {anchored}",
            Part::Lines,
            Part::Nodes,
            Part::Rulings,
            Part::Anchored
        )
    }
}
