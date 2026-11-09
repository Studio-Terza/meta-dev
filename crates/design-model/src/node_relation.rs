use named_derive::Named;

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeRelation {
    #[name("refines")]
    Refines,
    #[name("supersedes")]
    Supersedes,
    #[name("consumes")]
    Consumes,
}
