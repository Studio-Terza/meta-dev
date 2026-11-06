mod lowering;
mod namespace;
mod planting;

use code_model::{Building, Model};
use lint_core::{Readout, SourceFile};

use lowering::Lowered;
use planting::{Listed, Planting, ReadFile};

#[derive(Debug)]
pub struct Reader;

impl Reader {
    #[must_use]
    pub fn read(readout: &Readout) -> Model {
        Self::read_files(readout.files())
    }

    #[must_use]
    pub fn read_files(files: &[SourceFile]) -> Model {
        let mut building = Building::default();
        let read: Vec<ReadFile<'_>> = files
            .iter()
            .flat_map(|file| {
                std::iter::once(Listed::from(file))
                    .chain(file.test_modules().iter().map(|module| Listed::from(module).tested()))
            })
            .map(|listed| listed.read(&mut building))
            .collect();
        let lowered: Lowered<'_> = Planting::new(&read).lowered();
        lowered.built(building)
    }
}
