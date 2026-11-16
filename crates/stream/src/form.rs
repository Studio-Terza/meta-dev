use grammar_message::{Depth, Number, Opened};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Form {
    Sentence,
    Item(Number, Depth),
    Bullet,
    Quote,
}

impl Opened for Form {
    const BULLET: Form = Form::Bullet;

    const QUOTE: Form = Form::Quote;

    fn item(number: Number, depth: Depth) -> Form {
        Form::Item(number, depth)
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
