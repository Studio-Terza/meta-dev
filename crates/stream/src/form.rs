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

pub(crate) trait Forms {
    const SENTENCE: Self;
    const ITEM: Self;
    const BULLET: Self;
    const QUOTE: Self;
}

impl Form {
    pub(crate) fn formed<F: Forms>(self) -> F {
        match self {
            Form::Sentence => F::SENTENCE,
            Form::Item(..) => F::ITEM,
            Form::Bullet => F::BULLET,
            Form::Quote => F::QUOTE,
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
