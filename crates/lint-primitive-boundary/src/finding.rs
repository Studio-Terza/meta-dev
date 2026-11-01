use std::fmt;

use crate::string_sites::TextForm;
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
            Finding::BarePrimitive { primitive, context } => {
                write!(formatter, "bare {primitive} in {context}; name what it counts with a newtype")
            }
            Finding::WireForm { wire, context } => {
                write!(formatter, "{wire} in {context} is a wire form; parse it into a type at the door")
            }
            Finding::UnnamedSize { context } => write!(
                formatter,
                "usize in {context} is neither a length nor an index; a count gets a newtype, a size says so in its name"
            ),
            Finding::BareText { form, context } => write!(
                formatter,
                "bare {form} in {context}; give the text a newtype, with Display at the render edge and FromStr at the parse edge"
            ),
            Finding::TextOutsideDoor { handler } => {
                write!(formatter, "{handler} handles text outside a door; parse in FromStr or render in Display")
            }
        }
    }
}
