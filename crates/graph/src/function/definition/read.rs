use lint_layout::{LineNumber, Name};

use super::operator::Operator;
use crate::boundary::{LiteralValue, ParamIndex};
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
    Literal(LiteralValue),
    Applied { operator: Operator, operands: Vec<Argument> },
    Opaque,
}

impl Argument {
    #[must_use]
    pub fn applied(operator: Operator, operands: Vec<Argument>) -> Argument {
        let opaque = operands.iter().find(|operand| matches!(operand, Argument::Opaque)).map(|_| ());
        opaque.map_or(Argument::Applied { operator, operands }, |()| Argument::Opaque)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Read {
    reader: Reader,
    parameter: ParamIndex,
    fields: Vec<Name>,
    arguments: Vec<Argument>,
    line: LineNumber,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unpassed {
    reader: Reader,
    parameter: ParamIndex,
    fields: Vec<Name>,
    line: LineNumber,
}

impl Unpassed {
    #[must_use]
    pub fn new(reader: Reader, parameter: ParamIndex, fields: Vec<Name>, line: LineNumber) -> Unpassed {
        Unpassed { reader, parameter, fields, line }
    }

    #[must_use]
    pub fn passing(self, arguments: Vec<Argument>) -> Read {
        let Unpassed { reader, parameter, fields, line } = self;
        Read { reader, parameter, fields, arguments, line }
    }
}

impl Read {
    #[must_use]
    pub fn comparable(&self) -> Option<&[Argument]> {
        let opaque = self.arguments.iter().find(|argument| matches!(argument, Argument::Opaque));
        opaque.map_or(Some(self.arguments.as_slice()), |_| None)
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
