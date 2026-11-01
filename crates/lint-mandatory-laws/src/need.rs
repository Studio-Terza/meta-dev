#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Need {
    Roundtrip,
    Law,
}

impl Need {
    pub(crate) fn fix(self) -> &'static str {
        match self {
            Need::Roundtrip => {
                "has a conversion pair (Display and FromStr, Serialize and Deserialize, encode and decode, or to_x and from_x); add a `laws::roundtrip!` in a test that names it"
            }
            Need::Law => "has a hand-written Ord or PartialEq; add a `laws::law!` in a test that names it",
        }
    }
}
