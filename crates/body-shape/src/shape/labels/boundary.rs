use std::fmt;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Code {
    Mark(char),
    Word(Arc<str>),
    Literal(Arc<str>),
    Erased,
    Slot(usize),
}

impl Code {
    pub(crate) fn word(text: &str) -> Code {
        Code::Word(Arc::from(text))
    }

    pub(crate) fn literal(&self) -> Code {
        match self {
            Code::Word(text) | Code::Literal(text) => Code::Literal(Arc::clone(text)),
            Code::Mark(_) | Code::Erased | Code::Slot(_) => self.clone(),
        }
    }
}

impl fmt::Display for Code {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Code::Mark(mark) => write!(formatter, "{mark}"),
            Code::Word(text) | Code::Literal(text) => formatter.write_str(text),
            Code::Erased => formatter.write_str("_"),
            Code::Slot(slot) => write!(formatter, "${slot}"),
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
