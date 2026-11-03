use std::path::{Path, PathBuf};

use lint_layout::Name;
use lint_layout::Prefix;
use lint_layout::Word;

use crate::survey::Survey;

#[derive(Clone, Copy, Debug)]
pub enum Ignored {
    Hidden,
    VersionControl,
    BuildOutput,
}

impl Ignored {
    pub const FOR_PACKAGES: [Ignored; 2] = [Ignored::Hidden, Ignored::BuildOutput];
    pub const FOR_FILES: [Ignored; 2] = [Ignored::VersionControl, Ignored::BuildOutput];

    const BUILD_DIRECTORY: &'static str = "target";
    const MANIFEST: &'static str = "Cargo.toml";
    const HIDDEN: Word<'static> = Word::new(".");
    const VERSION_CONTROL: &'static str = ".git";

    pub fn manifests<'path>(directories: impl Iterator<Item = &'path Path>) -> Vec<PathBuf> {
        directories
            .filter(|directory| directory.file_name().is_some_and(|name| name == Self::BUILD_DIRECTORY))
            .filter_map(Path::parent)
            .map(|parent| parent.join(Self::MANIFEST))
            .collect()
    }

    pub fn applying_to(self, survey: &Survey, path: &Path) -> Option<Ignored> {
        let named = path.file_name();
        let applies = match self {
            Ignored::Hidden => named.is_some_and(|name| Name::from(name).matches::<Prefix>(&[Self::HIDDEN]).is_some()),
            Ignored::VersionControl => named.is_some_and(|name| name == Self::VERSION_CONTROL),
            Ignored::BuildOutput => {
                named.is_some_and(|name| name == Self::BUILD_DIRECTORY)
                    && path
                        .parent()
                        .map(|parent| parent.join(Self::MANIFEST))
                        .is_some_and(|manifest| survey.file(&manifest).is_some())
            }
        };
        applies.then_some(self)
    }
}
