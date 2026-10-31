use std::path::{Component, Path};

use crate::name::Name;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModulePath(Vec<Name>);

impl ModulePath {
    #[must_use]
    pub fn of_file(relative: &Path) -> ModulePath {
        let mut names: Vec<Name> = relative
            .with_extension("")
            .components()
            .filter_map(|component| match component {
                Component::Normal(segment) => Some(Name::from(segment)),
                _ => None,
            })
            .collect();
        if names.last().is_some_and(|last| *last == Name::from(std::ffi::OsStr::new("mod"))) {
            names.pop();
        }
        ModulePath(names)
    }

    #[must_use]
    pub fn child(&self, name: &Name) -> ModulePath {
        let mut names = self.0.clone();
        names.push(name.clone());
        ModulePath(names)
    }
}
