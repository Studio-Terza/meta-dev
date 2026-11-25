use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use file_ports::Kind;

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
        rules.iter().find_map(|rule| rule.matched(survey, path).map(|_| *rule))
    }

    fn matched<'path>(self, survey: &Survey, path: &'path Path) -> Option<&'path OsStr> {
        match self {
            Ignored::Hidden => {
                path.file_name().filter(|name| Name::from(*name).matches::<Prefix>(&[Self::HIDDEN]).is_some())
            }
            Ignored::VersionControl => path.file_name().filter(|name| *name == Self::VERSION_CONTROL),
            Ignored::BuildOutput => path
                .file_name()
                .filter(|name| *name == Self::BUILD_DIRECTORY)
                .filter(|_| Self::manifested(survey, path).is_some()),
        }
    }

    fn manifested(survey: &Survey, path: &Path) -> Option<Kind> {
        path.parent().map(|parent| parent.join(Self::MANIFEST)).and_then(|manifest| survey.file(&manifest))
    }
}
