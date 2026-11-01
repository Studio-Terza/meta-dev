use lint_placement::CrateFile;
use lint_placement::ModulePath;

use crate::source_file::SourceFile;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Home {
    crate_file: CrateFile,
    module: ModulePath,
}

impl Home {
    #[must_use]
    pub fn containing(file: &SourceFile) -> Home {
        Home { crate_file: file.crate_file().clone(), module: file.placement().module().clone() }
    }

    #[must_use]
    pub fn in_crate_with(&self, other: &Home) -> bool {
        self.crate_file == other.crate_file
    }
}
