use grammar::message::{Lexeme, Mark, Span};

use crate::form::Form;
use transcript_ports::Uuid;

use crate::ordinal::Ordinal;
use crate::segment::Segment;
use crate::segment_id::SegmentId;

#[derive(Clone, Copy, Debug)]
struct Part {
    form: Form,
    span: Span,
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
        Part { form: self.form, span: self.span.through(later.span) }
    }
}

impl Fill {
    fn marked(mark: Mark) -> Fill {
        match mark {
            Mark::Break => Fill::Blank,
            Mark::Stop | Mark::Numbered(..) | Mark::Bulleted | Mark::Quoted | Mark::Uuid | Mark::Plain => Fill::Written,
        }
    }

    fn after(self, mark: Mark) -> Fill {
        match self {
            Fill::Blank => Fill::marked(mark),
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

impl Form {
    fn stopping(self) -> Ending {
        match self {
            Form::Sentence => Ending::Close,
            Form::Item(..) | Form::Bullet | Form::Quote => Ending::Continue,
        }
    }
}

impl Open {
    fn starting(lexeme: Lexeme) -> Open {
        Open {
            part: Part { form: Form::opened_by(lexeme.mark()).unwrap_or(Form::Sentence), span: lexeme.span() },
            fill: Fill::marked(lexeme.mark()),
        }
    }

    fn extended(self, lexeme: Lexeme) -> Open {
        Open {
            part: self.part.joined(Part { form: self.part.form, span: lexeme.span() }),
            fill: self.fill.after(lexeme.mark()),
        }
    }

    fn ending_at(self, mark: Mark) -> Ending {
        match mark {
            Mark::Break => self.fill.breaking(),
            Mark::Stop => self.part.form.stopping(),
            Mark::Numbered(..) | Mark::Bulleted | Mark::Quoted | Mark::Uuid | Mark::Plain => Ending::Continue,
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
    pub fn segments(lexemes: Vec<Lexeme>, message: &Uuid) -> Vec<Segment> {
        lexemes
            .into_iter()
            .fold(Segmenter::default(), Segmenter::pushed)
            .closed()
            .done
            .into_iter()
            .enumerate()
            .map(|(position, part)| {
                Segment::new(SegmentId::new(message.clone(), Ordinal::new(position)), part.span, part.form)
            })
            .collect()
    }

    fn pushed(mut self, lexeme: Lexeme) -> Segmenter {
        let previous = self.open.take();
        let opening = Form::opened_by(lexeme.mark());
        let segmenter = opening.and(previous).into_iter().fold(self, Segmenter::record);
        let continuing = opening.map_or(previous, |_| None);
        let open = continuing.map_or_else(|| Open::starting(lexeme), |open| open.extended(lexeme));
        Segmenter { open: Some(open), ..segmenter }.ended(open.ending_at(lexeme.mark()))
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
