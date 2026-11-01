use std::fmt;

use crate::pair::Pair;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Need {
    Roundtrip(Pair),
    Law,
}

impl fmt::Display for Need {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Need::Roundtrip(pair) => write!(
                formatter,
                "has the conversion pair {pair} (Display and FromStr, Serialize and Deserialize, encode and decode, or to_x and from_x); add a `laws::roundtrip!` in a test of the same package that names the type and the pair"
            ),
            Need::Law => formatter.write_str(
                "has a hand-written Ord or PartialEq; add a `laws::law!` in a test of the same package that names it",
            ),
        }
    }
}
