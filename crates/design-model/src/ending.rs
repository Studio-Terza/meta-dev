use named_derive::Named;

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ending {
    #[name("dropped")]
    Dropped,
    #[name("superseded")]
    Superseded,
}
