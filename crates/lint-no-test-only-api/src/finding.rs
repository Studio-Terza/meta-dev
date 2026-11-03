use std::borrow::Cow;
use std::fmt;

use lint_core::Fix;
use lint_layout::Name;

#[derive(Debug)]
pub(crate) enum Finding<'name> {
    GatedOnTest,
    DocHidden,
    OnlyTests,
    Feature(Cow<'name, Name>),
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Gate {
    GatedOnTest,
    DocHidden,
}

impl From<Gate> for Finding<'_> {
    fn from(gate: Gate) -> Self {
        match gate {
            Gate::GatedOnTest => Finding::GatedOnTest,
            Gate::DocHidden => Finding::DocHidden,
        }
    }
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Finding::GatedOnTest => formatter.write_str("visible item gated on test"),
            Finding::DocHidden => formatter.write_str("#[doc(hidden)] on a visible item"),
            Finding::OnlyTests => formatter.write_str("visible item that only tests call, no shipped code names it"),
            Finding::Feature(name) => write!(formatter, "test feature {name}"),
        }
    }
}

impl lint_core::Finding for Finding<'_> {
    fn fix(&self) -> Fix {
        Fix::new(match self {
            Finding::GatedOnTest => "remove the gate and test through the real public API",
            Finding::DocHidden => "document it as real API, or make it private",
            Finding::OnlyTests => "remove it and test through the API shipped code uses",
            Finding::Feature(_) => "remove the feature and test through the API every build has",
        })
    }
}
