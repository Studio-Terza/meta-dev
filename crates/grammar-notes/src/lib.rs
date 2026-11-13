#![deny(clippy::wildcard_enum_match_arm)]

mod boundary;
mod cite;
mod date;
mod layout;
mod quote;
mod unclosed;

use named_derive::Named;

pub use boundary::{Entry, Line, Marking, Notes, Reading};
pub use cite::Cite;
pub use date::Date;
use layout::Layout;
pub use quote::Quote;
pub use unclosed::Unclosed;

#[derive(Named, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Ending {
    #[default]
    #[name("break")]
    Break,
    #[name("open")]
    Open,
}
