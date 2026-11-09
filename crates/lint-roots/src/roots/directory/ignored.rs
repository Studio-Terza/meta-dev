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

    pub fn manifests(directories: &[PathBuf]) -> Vec<PathBuf> {
        directories.iter().map(|directory| directory.join(Self::MANIFEST)).collect()
    }

    pub fn among(rules: &[Ignored], survey: &Survey, path: &Path) -> Option<Ignored> {
        rules.iter().find_map(|rule| rule.applying_to(survey, path))
    }

    pub fn applying_to(self, survey: &Survey, path: &Path) -> Option<Ignored> {
        match self {
            Ignored::Hidden => path
                .file_name()
                .map(Name::from)
                .filter(|name| name.matches::<Prefix>(&[Self::HIDDEN]).is_some())
                .map(|_| self),
            Ignored::VersionControl => path.file_name().filter(|name| *name == Self::VERSION_CONTROL).map(|_| self),
            Ignored::BuildOutput => path
                .file_name()
                .filter(|name| *name == Self::BUILD_DIRECTORY)
                .and_then(|_| path.parent())
                .map(|parent| parent.join(Self::MANIFEST))
                .filter(|manifest| survey.file(manifest).is_some())
                .map(|_| self),
        }
    }
}
