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
        formatter.write_str(match self {
            Relation::RepliesTo => "replies-to",
            Relation::Answers => "answers",
            Relation::Cites => "cites",
            Relation::Resends => "resends",
        })
    }
}
