use std::path::PathBuf;

use lint_core::{SourceFile, SourceText};
use lint_layout::Failure;
use lint_placement::Placement;

#[derive(Debug)]
pub struct Opened {
    shown: PathBuf,
    file: PathBuf,
    text: SourceText,
    placement: Placement,
}

impl Opened {
    #[must_use]
    pub fn new(shown: PathBuf, file: PathBuf, text: SourceText, placement: Placement) -> Opened {
        Opened { shown, file, text, placement }
    }

    pub fn parsed(&self) -> Result<SourceFile, Failure> {
        SourceFile::parse_at(self.shown.clone(), self.file.clone(), self.placement.clone(), self.text.clone())
    }
}
