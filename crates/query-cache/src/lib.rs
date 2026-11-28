mod boundary;
mod crud;
mod ports;
mod recorded;

pub use boundary::{CacheFile, Described, Entry, Unexpanded};
pub use crud::{Origin, Table};
pub use recorded::{Invoked, Placed, Recorded};

invariants::ignored! {
    pub(crate) enum Ignored {
        Unmapped => "a declared column type no Rust type spells is stored as unmapped, and the query names its type",
    }
}
