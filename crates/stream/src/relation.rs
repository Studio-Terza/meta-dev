use named_derive::Named;
use strum::VariantArray;

#[derive(Named, VariantArray, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
