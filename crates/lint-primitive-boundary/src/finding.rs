use std::fmt;

use crate::string_sites::TextForm;
use lint_core::Fix;
use lint_layout::Name;
use lint_layout::UsePath;

pub enum Finding {
    BarePrimitive { primitive: Name, context: Name },
    WireForm { wire: UsePath, context: Name },
    UnnamedSize { context: Name },
    TextOutsideDoor { handler: Name },
    BareText { form: TextForm, context: Name },
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Finding::BarePrimitive { primitive, context } => write!(formatter, "bare {primitive} in {context}"),
            Finding::WireForm { wire, context } => write!(formatter, "{wire} in {context} is a wire form"),
            Finding::UnnamedSize { context } => {
                write!(formatter, "usize in {context} is neither a length nor an index")
            }
            Finding::BareText { form, context } => write!(formatter, "bare {form} in {context}"),
            Finding::TextOutsideDoor { handler } => write!(formatter, "{handler} handles text outside a door"),
        }
    }
}

impl lint_core::Finding for Finding {
    fn fix(&self) -> Fix {
        Fix::new(match self {
            Finding::BarePrimitive { .. } => "name what it counts with a newtype",
            Finding::WireForm { .. } => "parse it into a type at the door",
            Finding::UnnamedSize { .. } => "a count gets a newtype, a size says so in its name",
            Finding::BareText { .. } => {
                "give the text a newtype, with Display at the render edge and FromStr at the parse edge"
            }
            Finding::TextOutsideDoor { .. } => "parse in FromStr or render in Display",
        })
    }
}
