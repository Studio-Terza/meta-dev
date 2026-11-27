mod memory;
mod rerooted;

pub use memory::Memory;
pub use rerooted::Rerooted;

invariants::ignored! {
    pub(crate) enum Ignored {
        BeyondRange => "a position past the usize range skips every byte",
        Unstored => "appending to a path with nothing stored starts from empty",
        Outside => "a held path outside the directory is not one of its entries, nor is it moved",
    }
}
