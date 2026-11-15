mod branches;
mod facts;
mod lowering;
mod namespace;
mod parallel;
mod planting;
mod root_reading;

use graph::Model;
use lint_core::{Readout, SourceFile};
use lint_layout::Crates;

pub use branches::{Branches, Propagation};
pub use facts::{FileFacts, GroupLowering, ListedGroup, Listing};
use lint_sweep::Sweep;
pub use lowering::ReceiverSyntax;
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
        let listing = Listing::of_files(files);
        let facts =
            listing.groups().iter().zip(files).flat_map(|(group, file)| Sweep::gathered(file, group.lowering()));
        FileFacts::merged(facts.collect())
    }

    #[must_use]
    pub fn rooted(model: Model, crates: &Crates) -> Model {
        let roots = RootReading::read(&model, crates);
        model.with_roots(roots)
    }
}
