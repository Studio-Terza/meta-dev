use algebra::{Monoid, Semigroup};
use grammar_message::{Boundary, Lexeme, Span};

use crate::form::{Form, Forms};
use transcript_ports::{Body, Uuid};

use crate::ordinal::Ordinal;
use crate::segment::Segment;
use crate::segment_id::SegmentId;

#[derive(Clone, Copy, Debug)]
struct Part {
    form: Form,
    span: Span,
    ink: Option<Span>,
}

#[derive(Clone, Copy, Debug)]
enum Fill {
    Blank,
    Written,
}

#[derive(Clone, Copy, Debug)]
enum Ending {
    Close,
    Continue,
}

#[derive(Clone, Copy, Debug)]
struct Open {
    part: Part,
    fill: Fill,
}

#[derive(Debug, Default)]
struct Stepped {
    recorded: Vec<Open>,
    open: Option<Open>,
}

#[derive(Debug, Default)]
struct Placed {
    head: Option<Part>,
    parts: Vec<Part>,
}

#[derive(Debug)]
pub struct Segmenter;

impl Semigroup for Part {
    fn combine(self, later: Part) -> Part {
        Part { form: self.form, span: self.span.combine(later.span), ink: self.ink.combine(later.ink) }
    }
}

impl Semigroup for Fill {
    fn combine(self, later: Fill) -> Fill {
        match self {
            Fill::Blank => later,
            Fill::Written => Fill::Written,
        }
    }
}

impl Semigroup for Open {
    fn combine(self, later: Open) -> Open {
        Open { part: self.part.combine(later.part), fill: self.fill.combine(later.fill) }
    }
}

impl Semigroup for Placed {
    fn combine(self, later: Placed) -> Placed {
        let Placed { head, mut parts } = self;
        let last = parts.pop();
        let merged = last.map(|earlier| later.head.into_iter().fold(earlier, Part::combine));
        let unattached = last.map_or(later.head, |_| None);
        parts.extend(merged);
        parts.extend(later.parts);
        Placed { head: head.combine(unattached), parts }
    }
}

impl Monoid for Placed {}

impl From<Open> for Placed {
    fn from(open: Open) -> Placed {
        match open.fill {
            Fill::Blank => Placed { head: Some(open.part), parts: Vec::new() },
            Fill::Written => Placed { head: None, parts: vec![open.part] },
        }
    }
}

impl Part {
    fn inked(self) -> Span {
        self.ink.unwrap_or_else(|| self.span.emptied())
    }
}

impl Fill {
    fn marked(lexeme: Lexeme) -> Fill {
        lexeme.ink().map_or(Fill::Blank, |_| Fill::Written)
    }

    fn breaking(self) -> Ending {
        match self {
            Fill::Blank => Ending::Continue,
            Fill::Written => Ending::Close,
        }
    }
}

impl Forms for Ending {
    const SENTENCE: Ending = Ending::Close;
    const ITEM: Ending = Ending::Continue;
    const BULLET: Ending = Ending::Continue;
    const QUOTE: Ending = Ending::Continue;
}

impl Ending {
    fn stepped(self, recorded: Option<Open>, current: Open) -> Stepped {
        match self {
            Ending::Close => Stepped { recorded: recorded.into_iter().chain([current]).collect(), open: None },
            Ending::Continue => Stepped { recorded: recorded.into_iter().collect(), open: Some(current) },
        }
    }
}

impl Open {
    fn starting(lexeme: Lexeme) -> Open {
        Open {
            part: Part {
                form: lexeme.mark().opened().unwrap_or(Form::Sentence),
                span: lexeme.span(),
                ink: lexeme.ink(),
            },
            fill: Fill::marked(lexeme),
        }
    }

    fn bounded(self, boundary: Boundary) -> Ending {
        match boundary {
            Boundary::Line => self.fill.breaking(),
            Boundary::Sentence => self.part.form.formed(),
        }
    }

    fn stepped(previous: Option<Open>, lexeme: Lexeme) -> Stepped {
        let opening: Option<Form> = lexeme.mark().opened();
        let recorded = opening.and(previous);
        let continuing = opening.map_or(previous, |_| None);
        let started = Open::starting(lexeme);
        let current = continuing.map_or(started, |open| open.combine(started));
        let ending = lexeme.mark().boundary().map_or(Ending::Continue, |boundary| current.bounded(boundary));
        ending.stepped(recorded, current)
    }
}

impl Segmenter {
    pub fn segments(lexemes: Vec<Lexeme>, message: &Uuid, body: &Body) -> Vec<Segment> {
        let scanned = lexemes.into_iter().fold(Stepped::default(), |scanned, lexeme| {
            let stepped = Open::stepped(scanned.open, lexeme);
            Stepped { recorded: scanned.recorded.combine(stepped.recorded), open: stepped.open }
        });
        let placed: Placed = Monoid::concat(scanned.recorded.into_iter().chain(scanned.open).map(Placed::from));
        body.numbered(placed.head.into_iter().chain(placed.parts).collect(), |place, part| {
            Segment::new(SegmentId::new(message.clone(), Ordinal::new(place)), part.inked(), part.form)
        })
    }
}
