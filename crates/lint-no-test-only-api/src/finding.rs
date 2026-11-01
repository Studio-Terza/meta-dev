use std::fmt;

use lint_core::Name;

#[derive(Clone, Debug)]
pub(crate) enum Finding {
    GatedOnTest,
    DocHidden,
    Feature(Name),
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Finding::GatedOnTest => {
                formatter.write_str("visible item gated on test; remove the gate and test through the real public API")
            }
            Finding::DocHidden => {
                formatter.write_str("#[doc(hidden)] on a visible item; document it as real API, or make it private")
            }
            Finding::Feature(name) => {
                write!(formatter, "test feature {name}; remove the feature and test through the API every build has")
            }
        }
    }
}
