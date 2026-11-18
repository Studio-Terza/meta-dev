use named_derive::Named;

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeRelation {
    Refines,
    Supersedes,
    Consumes,
}
