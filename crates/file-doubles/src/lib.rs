mod memory;

pub use memory::Memory;

invariants::ignored! {
    pub(crate) enum Ignored {
        BeyondRange => "a position past the usize range skips every byte",
        Unstored => "appending to a path with nothing stored starts from empty",
        Outside => "a held path outside the directory is not one of its entries",
    }
}
