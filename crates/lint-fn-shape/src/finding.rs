use std::fmt;

use lint_core::Bound;
use lint_core::Fix;

pub enum Finding {
    DeepNesting { most: Bound },
    WideSignature { most: Bound },
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
        match self {
            Finding::DeepNesting { .. } => Fix::new("lift the inner part into a method"),
            Finding::WideSignature { .. } => Fix::new("group the ones that travel together into a struct"),
            Finding::BooleanLocal => Fix::new("inline a one-use condition or make a mode a type"),
            Finding::TupleLocal => Fix::new("name it as a struct"),
            Finding::TupleAccumulator => Fix::new("give the accumulator a type"),
        }
    }
}
