use named_derive::Named;

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Specificity {
    General,
    Specific,
    Exact,
}
