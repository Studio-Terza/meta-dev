use std::fmt;

use lint_core::Fix;
use lint_layout::Name;
use lint_layout::UsePath;

pub enum Finding {
    BarePrimitive { primitive: Name, context: Name },
    WireForm { wire: UsePath, context: Name },
    TextOutsideDoor { handler: Name },
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BarePrimitive { primitive, context } => write!(formatter, "bare {primitive} in {context}"),
            Self::WireForm { wire, context } => write!(formatter, "{wire} in {context} is a wire form"),
            Self::TextOutsideDoor { handler } => write!(formatter, "{handler} handles text outside a door"),
        }
    }
}

impl lint_core::Finding for Finding {
    fn fix(&self) -> Fix {
        match self {
            Self::BarePrimitive { .. } => Fix::new(
                "take and return a domain type (a newtype, an enum or a proof type), or move the code into a module named boundary",
            ),
            Self::WireForm { .. } => Fix::new("parse it into a type at the door"),
            Self::TextOutsideDoor { .. } => Fix::new("parse in FromStr or render in Display"),
        }
    }
}
