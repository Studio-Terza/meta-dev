mod package;

use std::collections::BTreeSet;
use std::str::FromStr;

use serde::Deserialize;

use super::scope::Scope;
use crate::cli::command::failure::Failure;
use package::Package;

#[derive(Debug, Deserialize)]
pub(super) struct Workspace {
    packages: Vec<Package>,
    #[serde(default)]
    workspace_default_members: Vec<String>,
}

impl Workspace {
    const LIBRARIES: [&'static str; 6] = ["lib", "rlib", "dylib", "cdylib", "staticlib", "proc-macro"];
    const UNITS: [(&'static str, &'static [&'static str]); 4] =
        [("--lib", &Self::LIBRARIES), ("--bins", &["bin"]), ("--examples", &["example"]), ("--benches", &["bench"])];
    const TESTS: &'static str = "--tests";

    fn selected(&self, scope: &Scope) -> impl Iterator<Item = &Package> {
        self.packages.iter().filter(move |package| match scope {
            Scope::Workspace => true,
            Scope::Default => package.among(&self.workspace_default_members),
            Scope::Packages(names) => package.named(names),
        })
    }

    pub(super) fn targets(&self, scope: &Scope) -> Vec<String> {
        let kinds: BTreeSet<&str> = self.selected(scope).flat_map(Package::kinds).collect();
        Self::UNITS
            .iter()
            .filter(|(_, wanted)| wanted.iter().any(|kind| kinds.contains(kind)))
            .map(|(flag, _)| *flag)
            .chain([Self::TESTS])
            .map(str::to_owned)
            .collect()
    }
}

impl FromStr for Workspace {
    type Err = Failure<'static>;

    fn from_str(text: &str) -> Result<Workspace, Failure<'static>> {
        serde_json::from_str(text).map_err(Failure::Metadata)
    }
}
