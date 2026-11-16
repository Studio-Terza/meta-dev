use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Code {
    Mark(char),
    Word(Arc<str>),
    Literal(Arc<str>),
    Erased,
    New,
    Distance(usize),
}

impl Code {
    pub(crate) fn word(text: &str) -> Code {
        Code::Word(Arc::from(text))
    }

    pub(crate) fn literal(&self) -> Code {
        match self {
            Code::Word(text) | Code::Literal(text) => Code::Literal(Arc::clone(text)),
            Code::Mark(_) | Code::Erased | Code::New | Code::Distance(_) => self.clone(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Labels(Vec<Code>);

impl Labels {
    pub(crate) fn pushed(mut self, code: Code) -> Labels {
        self.0.push(code);
        self
    }

    pub(crate) fn count(&self) -> usize {
        self.0.len()
    }
}
