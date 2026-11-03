use std::fmt;

use lint_core::Fix;

use super::Pair;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Need<'facts> {
    Roundtrip(Pair<'facts>),
    Law,
}

impl fmt::Display for Need<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Need::Roundtrip(pair) => write!(
                formatter,
                "has the conversion pair {pair} (Display and FromStr, Serialize and Deserialize, encode and decode, or to_x and from_x)"
            ),
            Need::Law => formatter.write_str("has a hand-written Ord or PartialEq"),
        }
    }
}

impl lint_core::Finding for Need<'_> {
    fn fix(&self) -> Fix {
        match self {
            Need::Roundtrip(_) => {
                Fix::new("add a `laws::roundtrip!` in a test of the same package that names the type and the pair")
            }
            Need::Law => Fix::new("add a `laws::law!` in a test of the same package that names it"),
        }
    }
}
