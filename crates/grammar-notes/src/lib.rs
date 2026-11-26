#![deny(clippy::wildcard_enum_match_arm)]

mod boundary;
mod cite;
mod date;
mod quote;
mod unclosed;

use newtype::Listed;
use strum::{Display, IntoStaticStr, VariantArray};

pub use boundary::{Entry, Line, Marking, Notes, Reading};
pub use cite::Cite;
pub use date::Date;
pub use quote::Quote;
pub use unclosed::Unclosed;

#[derive(Display, IntoStaticStr, VariantArray, Listed, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum Ending {
    #[default]
    Break,
    Open,
}
