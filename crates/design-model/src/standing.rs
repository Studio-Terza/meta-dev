use named_derive::Named;

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Standing {
    #[name("current")]
    Current,
    #[name("superseded")]
    Superseded,
}
