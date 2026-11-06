use named_derive::Named;
use strum::VariantArray;

#[derive(Named, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SegmentRelation {
    #[name("answers")]
    Answers,
    #[name("refers-to")]
    RefersTo,
    #[name("supersedes")]
    Supersedes,
    #[name("corrects")]
    Corrects,
}
