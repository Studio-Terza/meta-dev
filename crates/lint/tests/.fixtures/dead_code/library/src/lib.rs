mod inner;

pub use inner::{Named, Unnamed};

pub struct Used;
pub struct Forgotten;
pub(crate) struct Crated;
struct Private;
pub enum Choice {
    One,
}
pub trait Shared {}
pub const LIMIT: u8 = 1;
pub static NAME: &str = "library";
pub type Alias = Used;
pub mod open {}
pub fn called() {}
pub fn uncalled() {}
impl Used {
    pub fn method(&self) {}
    pub fn idle(&self) {}
    pub const SIZE: u8 = 2;
    fn private(&self) {}
}
impl Shared for Used {}
#[cfg(test)]
pub fn gated() {}
macro_rules! listed {
    () => {};
}
