use named_derive::Named;

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[named(kebab)]
pub enum Relation {
    RepliesTo,
    Answers,
    Cites,
    Resends,
}
