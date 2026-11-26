use std::ops::Range;

use lint_layout::{LineNumber, Name};
use newtype::proptest::arbitrary::{Arbitrary, any};
use newtype::proptest::collection;
use newtype::proptest::prop_oneof;
use newtype::proptest::strategy::{BoxedStrategy, Just, Strategy};

use super::operator::Operator;
use crate::boundary::{LiteralValue, ParamIndex};
use crate::resolved::Resolved;

#[derive(newtype::Arbitrary, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

impl Argument {
    const DEPTH: u32 = 3;
    const NODES: u32 = 12;
    const BRANCHES: u32 = 3;
    const SAMPLED: Range<usize> = 0..3;

    fn grown(inner: BoxedStrategy<Argument>) -> BoxedStrategy<Argument> {
        Strategy::prop_map((any::<Operator>(), collection::vec(inner, Argument::SAMPLED)), |(operator, operands)| {
            Argument::applied(operator, operands)
        })
        .boxed()
    }
}

impl Arbitrary for Argument {
    type Parameters = ();
    type Strategy = BoxedStrategy<Argument>;

    fn arbitrary_with((): ()) -> BoxedStrategy<Argument> {
        let parameter = (any::<ParamIndex>(), collection::vec(any::<Name>(), Argument::SAMPLED))
            .prop_map(|(parameter, fields)| Argument::Parameter { parameter, fields });
        prop_oneof![
            parameter,
            any::<Reader>().prop_map(Argument::Named),
            any::<LiteralValue>().prop_map(Argument::Literal),
            Just(Argument::Opaque),
        ]
        .prop_recursive(Argument::DEPTH, Argument::NODES, Argument::BRANCHES, Argument::grown)
        .boxed()
    }
}

#[derive(newtype::Arbitrary, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[newtype(generate = Read::sampled)]
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
    const SAMPLED: Range<usize> = 0..3;

    fn sampled() -> impl Strategy<Value = Read> {
        Strategy::prop_map(
            (
                any::<Reader>(),
                any::<ParamIndex>(),
                collection::vec(any::<Name>(), Read::SAMPLED),
                any::<LineNumber>(),
                collection::vec(any::<Argument>(), Read::SAMPLED),
            ),
            |(reader, parameter, fields, line, arguments)| {
                Unpassed::new(reader, parameter, fields, line).passing(arguments)
            },
        )
    }

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
