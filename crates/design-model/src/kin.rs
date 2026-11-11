use named_derive::Named;

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kin {
    #[name("ancestor")]
    Ancestor,
    #[name("child")]
    Child,
}
