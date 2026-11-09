pub(crate) mod boundary;

pub use super::count::Count;
pub(crate) use boundary::Key;
pub use boundary::{Anchored, Attempted, Flailing, Lapsed, Line, Remark, Summary};
