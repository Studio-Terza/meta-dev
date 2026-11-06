use super::Number;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    Break,
    Stop,
    Numbered(Number),
    Bulleted,
    Quoted,
    Uuid,
    Plain,
}
