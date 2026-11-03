use std::fmt;

use lint_core::Fix;
use lint_core::Most;

pub enum Finding {
    DeepNesting { most: Most },
    WideSignature { most: Most },
    BooleanLocal,
    TupleLocal,
    TupleAccumulator,
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Finding::DeepNesting { most } => write!(formatter, "control flow nested deeper than {most} levels"),
            Finding::WideSignature { most } => write!(formatter, "more than {most} parameters"),
            Finding::BooleanLocal => formatter.write_str("boolean local"),
            Finding::TupleLocal => formatter.write_str("tuple literal bound to a local"),
            Finding::TupleAccumulator => formatter.write_str("tuple accumulator in a fold"),
        }
    }
}

impl lint_core::Finding for Finding {
    fn fix(&self) -> Fix {
        Fix::new(match self {
            Finding::DeepNesting { .. } => "lift the inner part into a method",
            Finding::WideSignature { .. } => "group the ones that travel together into a struct",
            Finding::BooleanLocal => "inline a one-use condition or make a mode a type",
            Finding::TupleLocal => "name it as a struct",
            Finding::TupleAccumulator => "give the accumulator a type",
        })
    }
}
