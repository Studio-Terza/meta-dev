use grammar::message::{Mark, Number};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Form {
    Sentence,
    Item(Number),
    Bullet,
    Quote,
}

impl Form {
    pub(crate) fn opened_by(mark: Mark) -> Option<Form> {
        match mark {
            Mark::Numbered(number) => Some(Form::Item(number)),
            Mark::Bulleted => Some(Form::Bullet),
            Mark::Quoted => Some(Form::Quote),
            Mark::Break | Mark::Stop | Mark::Uuid | Mark::Plain => None,
        }
    }
}

impl Form {
    pub(crate) fn number(self) -> Option<Number> {
        match self {
            Form::Item(number) => Some(number),
            Form::Sentence | Form::Bullet | Form::Quote => None,
        }
    }
}
