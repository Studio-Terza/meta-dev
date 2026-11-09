use lint_placement::CrateFile;
use lint_placement::ModulePath;

use crate::source_file::SourceFile;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Home<'file> {
    crate_file: &'file CrateFile,
    module: &'file ModulePath,
}

impl<'file> Home<'file> {
    pub const NOWHERE: Home<'static> = Home { crate_file: &CrateFile::UNPLACED, module: &ModulePath::ROOT };

    #[must_use]
    pub fn containing(file: &'file SourceFile) -> Home<'file> {
        Home { crate_file: file.crate_file(), module: file.placement().module() }
    }

    #[must_use]
    pub fn in_crate_with(self, other: Home<'_>) -> Option<Home<'_>> {
        (self.crate_file == other.crate_file).then_some(other)
    }
}
