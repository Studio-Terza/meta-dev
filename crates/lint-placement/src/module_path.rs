use std::ffi::OsStr;
use std::path::{Component, Path};

use lint_layout::Name;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModulePath(Vec<Name>);

impl ModulePath {
    const MODULE_FILE: &'static str = "mod";

    #[must_use]
    pub fn of_file(relative: &Path) -> ModulePath {
        let stem = relative.with_extension("");
        let directory = Some(stem.as_path())
            .filter(|module| module.file_name() != Some(OsStr::new(Self::MODULE_FILE)))
            .or_else(|| stem.parent());
        ModulePath(
            directory
                .into_iter()
                .flat_map(Path::components)
                .filter_map(|component| match component {
                    Component::Normal(segment) => Some(Name::from(segment)),
                    _ => None,
                })
                .collect(),
        )
    }

    #[must_use]
    pub fn segments(&self) -> &[Name] {
        &self.0
    }

    #[must_use]
    pub fn child(&self, name: &Name) -> ModulePath {
        ModulePath(self.0.iter().cloned().chain(Some(name.clone())).collect())
    }
}
