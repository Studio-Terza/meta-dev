mod boundary;
mod cache;
mod crud;
mod describe;
mod ports;
mod recorded;
mod sources;

use boundary::CacheFile;
pub use boundary::{Described, Entry, Unexpanded};
pub use cache::{Cache, Uncached};
pub use crud::{Origin, Table};
use describe::Describe;
pub use describe::{DescribeFiller, Undescribed, describe_ops};
pub use recorded::{Invoked, Placed, Recorded};

invariants::ignored! {
    pub(crate) enum Ignored {
        Unmapped => "a declared column type no Rust type spells is stored as unmapped, and the query names its type",
    }
}
