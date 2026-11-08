mod lowering;
mod namespace;
mod planting;
mod root_reading;

use code_model::{Building, Model};
use lint_core::{Readout, SourceFile};
use lint_layout::Crates;

use lowering::Lowered;
use planting::{Listed, Planting, ReadFile};
use root_reading::RootReading;

#[derive(Debug)]
pub struct Reader;

impl Reader {
    #[must_use]
    pub fn read(readout: &Readout, crates: &Crates) -> Model {
        let model = Self::read_files(readout.files());
        let roots = RootReading::read(&model, crates);
        model.with_roots(roots)
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
