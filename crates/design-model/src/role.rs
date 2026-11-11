use named_derive::Named;

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    #[name("core")]
    Core,
    #[name("secondary")]
    Secondary,
}
