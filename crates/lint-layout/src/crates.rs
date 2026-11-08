use std::collections::BTreeMap;
use std::sync::Arc;

use crate::dependencies::Dependencies;
use crate::package_name::PackageName;
use crate::source_text::SourceText;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LibraryForm {
    Rust,
    ProcMacro,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrateSource {
    root: Option<SourceText>,
    form: LibraryForm,
    dependencies: Dependencies,
}

impl CrateSource {
    #[must_use]
    pub fn new(root: Option<SourceText>, form: LibraryForm, dependencies: Dependencies) -> CrateSource {
        CrateSource { root, form, dependencies }
    }

    #[must_use]
    pub fn form(&self) -> LibraryForm {
        self.form
    }

    #[must_use]
    pub fn root(&self) -> Option<&SourceText> {
        self.root.as_ref()
    }

    #[must_use]
    pub fn dependencies(&self) -> &Dependencies {
        &self.dependencies
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Crates(Arc<BTreeMap<PackageName, CrateSource>>);

impl Crates {
    #[must_use]
    pub fn source(&self, package: &PackageName) -> Option<&CrateSource> {
        self.0.get(package)
    }
}

impl FromIterator<(PackageName, CrateSource)> for Crates {
    fn from_iter<Found: IntoIterator<Item = (PackageName, CrateSource)>>(found: Found) -> Crates {
        let first = found.into_iter().fold(BTreeMap::new(), |mut first, (package, source)| {
            first.entry(package).or_insert(source);
            first
        });
        Crates(Arc::new(first))
    }
}
