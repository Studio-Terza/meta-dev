use named_derive::Named;

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
#[named(kebab)]
pub enum SegmentRelation {
    Answers,
    RefersTo,
    Supersedes,
    Corrects,
}
