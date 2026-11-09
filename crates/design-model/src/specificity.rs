use named_derive::Named;

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Specificity {
    #[name("general")]
    General,
    #[name("specific")]
    Specific,
    #[name("exact")]
    Exact,
}
