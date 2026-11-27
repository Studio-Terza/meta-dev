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
pub fn called() -> Returned {
    Returned { kept: Kept }
}
pub fn uncalled() -> Stranded {
    Stranded
}
pub struct Returned {
    pub kept: Kept,
    hidden: Hidden,
}
pub struct Kept;
pub fn spelled_late() {}
pub struct Hidden;
pub struct Stranded;
pub struct Refused;
impl Used {
    pub fn method(&self) {}
    pub fn idle(&self) {}
    pub const SIZE: u8 = 2;
    fn private(&self) {}
}
impl Shared for Used {}
pub trait Bounded {}
impl<Held: Bounded> Shared for Vec<Held> {}
impl std::str::FromStr for Used {
    type Err = Refused;
    fn from_str(_: &str) -> Result<Used, Refused> {
        Ok(Used)
    }
}
#[cfg(test)]
pub fn gated() {}
macro_rules! listed {
    () => {};
}
