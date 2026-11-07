use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Via {
    Field,
    Variant,
    Bound,
}

impl fmt::Display for Via {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Field => formatter.write_str("field"),
            Self::Variant => formatter.write_str("variant"),
            Self::Bound => formatter.write_str("bound"),
        }
    }
}
