use std::collections::BTreeSet;
use std::fmt;

use lint_core::{Finding, Fix};
use lint_layout::{LineNumber, TableKey};
use lint_placement::PackageRoot;
use workspace_metadata::{Breaching, Reformatted, Unleafed, Unported, Unroled};

pub(crate) trait Fixed<'meta>: fmt::Display + AsRef<Breaching<'meta>> {
    const FIX: Fix;

    fn lines(&self, package: &PackageRoot) -> BTreeSet<LineNumber> {
        package.dependency_sites().sites(self.as_ref().keys())
    }
}

#[derive(Debug)]
pub(super) struct Breached<'breach, Breach>(&'breach Breach);

impl<'breach, Breach> Breached<'breach, Breach> {
    pub(super) fn new(breach: &'breach Breach) -> Breached<'breach, Breach> {
        Breached(breach)
    }
}

impl<Breach: fmt::Display> fmt::Display for Breached<'_, Breach> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl<'meta, Breach: Fixed<'meta>> Finding for Breached<'_, Breach> {
    fn fix(&self) -> Fix {
        Breach::FIX
    }
}

impl<'meta> Fixed<'meta> for Unroled<'meta> {
    const FIX: Fix = Fix::new(
        "depend on the crates that declare the role and wrap the port, or make this crate one of them by declaring the role",
    );
}

impl<'meta> Fixed<'meta> for Unleafed<'meta> {
    const FIX: Fix = Fix::new(
        "keep the lexer a leaf: depend only on the logos port, on other lexers and on crates whose closure holds no package a port owns",
    );
}

impl<'meta> Fixed<'meta> for Reformatted<'meta> {
    const FIX: Fix = Fix::new("lex each format in one lexer crate: merge the two, or name the format each one lexes");

    fn lines(&self, package: &PackageRoot) -> BTreeSet<LineNumber> {
        package.key_lines().line(TableKey::LexerFormat).into_iter().collect()
    }
}

impl<'meta> Fixed<'meta> for Unported<'meta> {
    const FIX: Fix = Fix::new(
        "reach the owned package through a crate declaring the port's dependents role, or move the proc-macro into the owning unit",
    );
}
