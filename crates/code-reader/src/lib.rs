mod lowering;

use code_model::Model;
use lint_core::{Readout, SourceFile};

use lowering::Lowering;

#[derive(Debug)]
pub struct Reader;

impl Reader {
    #[must_use]
    pub fn read(readout: &Readout) -> Model {
        Self::read_files(readout.files())
    }

    #[must_use]
    pub fn read_files(files: &[SourceFile]) -> Model {
        files
            .iter()
            .flat_map(|file| std::iter::once(file).chain(file.test_modules()))
            .fold(Model::default(), |model, file| model.with_file(file.path().to_path_buf(), Lowering::lowered(file)))
    }
}
