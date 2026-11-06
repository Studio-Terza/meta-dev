use grammar::message::{Depth, Mark, Number};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Form {
    Sentence,
    Item(Number, Depth),
    Bullet,
    Quote,
}

impl Form {
    pub(crate) fn opened_by(mark: Mark) -> Option<Form> {
        match mark {
            Mark::Numbered(number, depth) => Some(Form::Item(number, depth)),
            Mark::Bulleted => Some(Form::Bullet),
            Mark::Quoted => Some(Form::Quote),
            Mark::Break | Mark::Stop | Mark::Uuid | Mark::Plain => None,
        }
    }
}

impl Form {
    pub(crate) fn top_level_number(self) -> Option<Number> {
        match self {
            Form::Item(number, Depth::Top) => Some(number),
            Form::Item(_, Depth::Nested) | Form::Sentence | Form::Bullet | Form::Quote => None,
        }
    }
}
