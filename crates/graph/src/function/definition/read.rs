use lint_layout::{LineNumber, Name};

use crate::boundary::ParamIndex;
use crate::resolved::Resolved;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Reader {
    Named(Resolved),
    Constructor(Resolved),
    Associated { owner: Resolved, name: Name },
}

impl Reader {
    #[must_use]
    pub fn resolved(&self) -> &Resolved {
        match self {
            Reader::Named(called) | Reader::Constructor(called) => called,
            Reader::Associated { owner, .. } => owner,
        }
    }

    #[must_use]
    pub fn member(&self) -> Option<&Name> {
        match self {
            Reader::Associated { name, .. } => Some(name),
            Reader::Named(_) | Reader::Constructor(_) => None,
        }
    }

    #[must_use]
    pub fn constructing(&self) -> Option<&Reader> {
        match self {
            Reader::Constructor(_) => Some(self),
            Reader::Named(_) | Reader::Associated { .. } => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Argument {
    Parameter { parameter: ParamIndex, fields: Vec<Name> },
    Named(Reader),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Read {
    reader: Reader,
    parameter: ParamIndex,
    fields: Vec<Name>,
    arguments: Vec<Argument>,
    line: LineNumber,
}

impl Read {
    #[must_use]
    pub fn new(reader: Reader, parameter: ParamIndex, fields: Vec<Name>, line: LineNumber) -> Read {
        Read { reader, parameter, fields, arguments: Vec::new(), line }
    }

    #[must_use]
    pub fn passing(self, arguments: Vec<Argument>) -> Read {
        Read { arguments, ..self }
    }

    #[must_use]
    pub fn arguments(&self) -> &[Argument] {
        &self.arguments
    }

    #[must_use]
    pub fn reader(&self) -> &Reader {
        &self.reader
    }

    #[must_use]
    pub fn parameter(&self) -> ParamIndex {
        self.parameter
    }

    #[must_use]
    pub fn fields(&self) -> &[Name] {
        &self.fields
    }
}

impl AsRef<LineNumber> for Read {
    fn as_ref(&self) -> &LineNumber {
        &self.line
    }
}
