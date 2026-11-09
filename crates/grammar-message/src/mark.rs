use super::{Depth, Number};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    Break,
    Stop,
    Numbered(Number, Depth),
    Bulleted,
    Quoted,
    Uuid,
    Plain,
}
