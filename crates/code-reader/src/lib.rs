mod facts;
mod lowering;
mod namespace;
mod parallel;
mod planting;
mod root_reading;

use code_model::Model;
use lint_core::{Readout, SourceFile};
use lint_layout::Crates;

pub use facts::{FileFacts, ListedGroup, Listing};
use root_reading::RootReading;

#[derive(Debug)]
pub struct Reader;

impl Reader {
    #[must_use]
    pub fn read(readout: &Readout, crates: &Crates) -> Model {
        Self::rooted(Self::read_files(readout.files()), crates)
    }

    #[must_use]
    pub fn read_files(files: &[SourceFile]) -> Model {
        FileFacts::merged(Listing::of_files(files).groups().iter().flat_map(ListedGroup::lowered).collect())
    }

    #[must_use]
    pub fn rooted(model: Model, crates: &Crates) -> Model {
        let roots = RootReading::read(&model, crates);
        model.with_roots(roots)
    }
}
