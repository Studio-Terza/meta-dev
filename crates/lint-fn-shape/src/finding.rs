use std::fmt;

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
            Finding::DeepNesting { most } => {
                write!(formatter, "control flow nested deeper than {most} levels; lift the inner part into a method")
            }
            Finding::WideSignature { most } => {
                write!(formatter, "more than {most} parameters; group the ones that travel together into a struct")
            }
            Finding::BooleanLocal => {
                formatter.write_str("boolean local; inline a one-use condition or make a mode a type")
            }
            Finding::TupleLocal => formatter.write_str("tuple literal bound to a local; name it as a struct"),
            Finding::TupleAccumulator => {
                formatter.write_str("tuple accumulator in a fold; give the accumulator a type")
            }
        }
    }
}
