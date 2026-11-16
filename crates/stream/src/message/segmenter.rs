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
pub struct Segmenter {
    done: Vec<Part>,
    open: Option<Open>,
}

impl Part {
    fn joined(self, later: Part) -> Part {
        let ink = self.ink.map_or(later.ink, |ink| Some(later.ink.map_or(ink, |last| ink.through(last))));
        Part { form: self.form, span: self.span.through(later.span), ink }
    }

    fn inked(self) -> Span {
        self.ink.unwrap_or_else(|| self.span.emptied())
    }
}

impl Fill {
    fn marked(lexeme: Lexeme) -> Fill {
        lexeme.ink().map_or(Fill::Blank, |_| Fill::Written)
    }

    fn after(self, lexeme: Lexeme) -> Fill {
        match self {
            Fill::Blank => Fill::marked(lexeme),
            Fill::Written => Fill::Written,
        }
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

    fn extended(self, lexeme: Lexeme) -> Open {
        Open {
            part: self.part.joined(Part { form: self.part.form, span: lexeme.span(), ink: lexeme.ink() }),
            fill: self.fill.after(lexeme),
        }
    }

    fn bounded(self, boundary: Boundary) -> Ending {
        match boundary {
            Boundary::Line => self.fill.breaking(),
            Boundary::Sentence => self.part.form.formed(),
        }
    }

    fn placed(self, previous: Option<Part>) -> Vec<Part> {
        match self.fill {
            Fill::Blank => previous.map_or_else(|| vec![self.part], |earlier| vec![earlier.joined(self.part)]),
            Fill::Written => previous.into_iter().chain(Some(self.part)).collect(),
        }
    }
}

impl Segmenter {
    pub fn segments(lexemes: Vec<Lexeme>, message: &Uuid, body: &Body) -> Vec<Segment> {
        let parts = lexemes.into_iter().fold(Segmenter::default(), Segmenter::pushed).closed().done;
        body.numbered(parts, |place, part| {
            Segment::new(SegmentId::new(message.clone(), Ordinal::new(place)), part.inked(), part.form)
        })
    }

    fn pushed(mut self, lexeme: Lexeme) -> Segmenter {
        let previous = self.open.take();
        let opening: Option<Form> = lexeme.mark().opened();
        let segmenter = opening.and(previous).into_iter().fold(self, Segmenter::record);
        let continuing = opening.map_or(previous, |_| None);
        let open = continuing.map_or_else(|| Open::starting(lexeme), |open| open.extended(lexeme));
        Segmenter { open: Some(open), ..segmenter }
            .ended(lexeme.mark().boundary().map_or(Ending::Continue, |boundary| open.bounded(boundary)))
    }

    fn ended(self, ending: Ending) -> Segmenter {
        match ending {
            Ending::Close => self.closed(),
            Ending::Continue => self,
        }
    }

    fn closed(mut self) -> Segmenter {
        let open = self.open.take();
        open.into_iter().fold(self, Segmenter::record)
    }

    fn record(mut self, open: Open) -> Segmenter {
        let previous = self.done.pop();
        self.done.extend(open.placed(previous));
        self
    }
}
