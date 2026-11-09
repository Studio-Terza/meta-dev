mod boundary;
mod cite;
mod layout;
mod quote;
pub mod stamp;
mod unclosed;

use named_derive::Named;
use stamp::{Clock, Date};

pub use boundary::{Entry, Line, Notes, Reading};
pub use cite::Cite;
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
