mod boundary;
mod disk;
mod file_failure;
mod files;
mod kind;
mod offset;
mod tail;

pub use boundary::{Bytes, Epoch, Size, Stamp, Stamps, Text};
pub use disk::Disk;
pub use file_failure::FileFailure;
pub use files::{Batched, Files, FilesFiller, files_ops};
pub use kind::Kind;
pub use offset::Offset;
pub use tail::Tail;

invariants::ignored! {
    pub(crate) enum Ignored {
        BeyondRange => "a length or position past the target integer's range saturates, or advances nothing",
        Unformattable => "a Display that fails to write yields empty bytes",
        Unreadable => "metadata that cannot be read reads as a missing path",
        BeforeEpoch => "a file time before 1970 reads as the epoch itself",
    }
}
