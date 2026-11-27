mod boundary;
mod memory;
mod scripted;
mod stepped;
mod variables;

pub use boundary::Count;
pub use memory::Memory;
pub use scripted::Scripted;
pub use stepped::{Oversleep, Stepped};
pub use variables::Variables;

invariants::ignored! {
    pub(crate) enum Ignored {
        BeyondRange => "a position past the usize range skips every byte",
        Unstored => "appending to a path with nothing stored starts from empty",
        Outside => "a held path outside the directory is not one of its entries",
    }
}
