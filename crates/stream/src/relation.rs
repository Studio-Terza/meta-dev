use named_derive::Named;

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Relation {
    #[name("replies-to")]
    RepliesTo,
    #[name("answers")]
    Answers,
    #[name("cites")]
    Cites,
    #[name("resends")]
    Resends,
}
