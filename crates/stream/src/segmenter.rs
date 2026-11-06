use grammar::message::{Lexeme, Mark, Span};

use crate::Form;

#[derive(Clone, Copy, Debug)]
pub struct Part {
    pub form: Form,
    pub span: Span,
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

impl Fill {
    fn of(mark: Mark) -> Fill {
        match mark {
            Mark::Break => Fill::Blank,
            Mark::Stop | Mark::Numbered(_) | Mark::Bulleted | Mark::Quoted | Mark::Uuid | Mark::Plain => Fill::Written,
        }
    }

    fn after(self, mark: Mark) -> Fill {
        match self {
            Fill::Blank => Fill::of(mark),
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
            Form::Item(_) | Form::Bullet | Form::Quote => Ending::Continue,
        }
    }
}

impl Open {
    fn starting(lexeme: Lexeme) -> Open {
        Open {
            part: Part { form: Form::opened_by(lexeme.mark).unwrap_or(Form::Sentence), span: lexeme.span },
            fill: Fill::of(lexeme.mark),
        }
    }

    fn extended(self, lexeme: Lexeme) -> Open {
        Open {
            part: Part { form: self.part.form, span: Span { start: self.part.span.start, end: lexeme.span.end } },
            fill: self.fill.after(lexeme.mark),
        }
    }

    fn ending_at(self, mark: Mark) -> Ending {
        match mark {
            Mark::Break => self.fill.breaking(),
            Mark::Stop => self.part.form.stopping(),
            Mark::Numbered(_) | Mark::Bulleted | Mark::Quoted | Mark::Uuid | Mark::Plain => Ending::Continue,
        }
    }
}

impl Segmenter {
    pub fn parts(lexemes: Vec<Lexeme>) -> Vec<Part> {
        lexemes.into_iter().fold(Segmenter::default(), Segmenter::pushed).closed().done
    }

    fn pushed(mut self, lexeme: Lexeme) -> Segmenter {
        let previous = self.open.take();
        let opening = Form::opened_by(lexeme.mark);
        let segmenter = opening.and(previous).into_iter().fold(self, Segmenter::record);
        let continuing = opening.map_or(previous, |_| None);
        let open = continuing.map_or_else(|| Open::starting(lexeme), |open| open.extended(lexeme));
        Segmenter { open: Some(open), ..segmenter }.ended(open.ending_at(lexeme.mark))
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
        let attached = Some(open)
            .filter(|blank| matches!(blank.fill, Fill::Blank))
            .and_then(|blank| self.done.last_mut().map(|last| last.span.end = blank.part.span.end));
        attached.unwrap_or_else(|| self.done.push(open.part));
        self
    }
}
