use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Relation {
    RepliesTo,
    Answers,
    Cites,
    Resends,
}

impl fmt::Display for Relation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Relation::RepliesTo => formatter.write_str("replies-to"),
            Relation::Answers => formatter.write_str("answers"),
            Relation::Cites => formatter.write_str("cites"),
            Relation::Resends => formatter.write_str("resends"),
        }
    }
}
