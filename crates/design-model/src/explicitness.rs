use named_derive::Named;

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Explicitness {
    #[name("explicit")]
    Explicit,
    #[name("implicit")]
    Implicit,
}
