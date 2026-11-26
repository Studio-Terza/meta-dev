mod absent;
mod bytes;
mod cached;
mod disk;
mod file_failure;
mod files;
mod kind;
mod memory;
mod offset;
mod size;
mod stamps;
mod tail;
mod text;

pub use absent::Absent;
pub use bytes::Bytes;
pub use cached::Cached;
pub use disk::Disk;
pub use file_failure::FileFailure;
pub use files::Files;
pub use kind::Kind;
pub use memory::Memory;
pub use offset::Offset;
pub use size::Size;
pub use stamps::{Epoch, Stamp, Stamps};
pub use tail::Tail;
pub use text::Text;

invariants::ignored! {
    pub(crate) enum Ignored {
        BeyondRange => "a length or position past the target integer's range saturates, or advances nothing",
        Unformattable => "a Display that fails to write yields empty bytes",
        Undecoded => "bytes that fail to read or decode as UTF-8 fall through to the backing files' own read_text",
        KindRecalled => "a kind is never remembered as failed; were one, the backing files answer afresh",
        Unreadable => "metadata that cannot be read reads as a missing path",
        Unstored => "appending to a path with nothing stored starts from empty",
        Outside => "a held path outside the directory is not one of its entries",
        BeforeEpoch => "a file time before 1970 reads as the epoch itself",
    }
}
