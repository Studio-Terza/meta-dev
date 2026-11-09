use std::fmt;

use design_model::{NodeId, NotesLine, Prefix, Provenance};
use grammar_notes::{Cite, Quote};
use grammar_output::field::{Bare, Id, Integer, Quoted, Word};
use grammar_output::{Fields, Unread};
use named_derive::Named;
use newtype::Newtype;
use strum::VariantArray;
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

#[derive(Named, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Miss {
    #[name("unheard")]
    Unheard,
    #[name("ambiguous")]
    Ambiguous,
    #[name("unclosed")]
    Unclosed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reconciliation {
    lines: Count,
    nodes: Count,
    rulings: Count,
    anchored: Count,
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
enum Part {
    #[name("line")]
    Line,
    #[name("quote")]
    Quote,
    #[name("cite")]
    Cite,
    #[name("lines")]
    Lines,
    #[name("nodes")]
    Nodes,
    #[name("rulings")]
    Rulings,
    #[name("anchored")]
    Anchored,
    #[name("segments")]
    Segments,
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Note {
    #[name("imported")]
    Imported,
    #[name("unresolved")]
    Unresolved,
    #[name("unanchored")]
    Unanchored,
    #[name("ambiguous-anchor")]
    AmbiguousAnchor,
    #[name("reconciled")]
    Reconciled,
    #[name("cites")]
    Cites,
}

impl Noted {
    pub(crate) fn read(note: Note, fields: &mut Fields) -> Result<Noted, Unread> {
        match note {
            Note::Imported => Ok(Noted::Imported {
                line: NotesLine::new(fields.past(&[Part::Line])?.field::<Integer<_>>()?),
                node: fields.field::<Id<_>>()?,
            }),
            Note::Unresolved => Ok(Noted::Unresolved {
                line: NotesLine::new(fields.past(&[Part::Line])?.field::<Integer<_>>()?),
                reference: Reference::read(fields)?,
                miss: fields.field::<Word<_>>()?,
            }),
            Note::Unanchored => {
                Ok(Noted::Unanchored(NotesLine::new(fields.past(&[Part::Line])?.field::<Integer<_>>()?)))
            }
            Note::AmbiguousAnchor => Ok(Noted::Ambiguous {
                line: NotesLine::new(fields.past(&[Part::Line])?.field::<Integer<_>>()?),
                cite: fields.past(&[Part::Cite])?.field::<Bare<_>>()?,
                segments: Count::new(fields.past(&[Part::Segments])?.field::<Integer<_>>()?),
            }),
            Note::Reconciled => Reconciliation::read(fields).map(Noted::Reconciled),
            Note::Cites => Ok(Noted::Cites {
                message: fields.field::<Bare<_>>()?,
                author: fields.field::<Word<_>>()?,
                entered: fields.past(&[Key::Entered])?.field::<Word<_>>()?,
            }),
        }
    }
}

impl Reference {
    fn read(fields: &mut Fields) -> Result<Reference, Unread> {
        match fields.field::<Word<Part>>()? {
            Part::Quote => Ok(Reference::Quote(fields.field::<Quoted<_>>()?)),
            Part::Cite => Ok(Reference::Cite(fields.field::<Bare<_>>()?)),
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

    fn read(fields: &mut Fields) -> Result<Reconciliation, Unread> {
        Ok(Reconciliation {
            lines: Count::new(fields.past(&[Part::Lines])?.field::<Integer<_>>()?),
            nodes: Count::new(fields.past(&[Part::Nodes])?.field::<Integer<_>>()?),
            rulings: Count::new(fields.past(&[Part::Rulings])?.field::<Integer<_>>()?),
            anchored: Count::new(fields.past(&[Part::Anchored])?.field::<Integer<_>>()?),
        })
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
