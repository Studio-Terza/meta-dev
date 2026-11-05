use std::ffi::OsStr;
use std::fmt;
use std::path::Path;

use lint_layout::Name;
use lint_placement::CrateName;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Unit<'found> {
    krate: &'found CrateName,
    binary: Option<&'found Path>,
}

impl<'found> Unit<'found> {
    const MAIN: &'static str = "main";
    const SOURCE: &'static str = "src";

    pub(crate) fn library(krate: &'found CrateName) -> Unit<'found> {
        Unit { krate, binary: None }
    }

    pub(crate) fn binary(krate: &'found CrateName, root: &'found Path) -> Unit<'found> {
        Unit { krate, binary: Some(root) }
    }

    pub(crate) fn krate(&self) -> &'found CrateName {
        self.krate
    }

    pub(crate) fn library_named(self, name: &Name) -> Option<Unit<'found>> {
        Some(self)
            .filter(|unit| unit.binary.is_none())
            .filter(|unit| std::borrow::Borrow::<Name>::borrow(unit.krate) == name)
    }
}

#[derive(Clone, Copy, Debug)]
struct BinaryName<'found> {
    krate: &'found CrateName,
    root: &'found Path,
}

impl fmt::Display for BinaryName<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let stem = self.root.file_stem().filter(|stem| *stem != OsStr::new(Unit::MAIN));
        let directory = self.root.parent().and_then(Path::file_name).filter(|name| *name != OsStr::new(Unit::SOURCE));
        let shown =
            stem.or(directory).map_or_else(|| self.krate.to_string(), |name| name.to_string_lossy().into_owned());
        write!(formatter, "{shown}")
    }
}

impl fmt::Display for Unit<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.binary {
            None => write!(formatter, "crate {}", self.krate),
            Some(root) => write!(formatter, "crate {} binary {}", self.krate, BinaryName { krate: self.krate, root }),
        }
    }
}
