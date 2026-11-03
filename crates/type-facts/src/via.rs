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
            Via::Field => formatter.write_str("field"),
            Via::Variant => formatter.write_str("variant"),
            Via::Bound => formatter.write_str("bound"),
        }
    }
}
