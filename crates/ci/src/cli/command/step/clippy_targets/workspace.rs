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
    const UNITS: [&'static str; 5] = ["--lib", "--bins", "--examples", "--benches", "--test"];
    const TEST: &'static str = "--test";

    fn selected(&self, scope: &Scope) -> impl Iterator<Item = &Package> {
        self.packages.iter().filter(move |package| match scope {
            Scope::Workspace => true,
            Scope::Default => package.among(&self.workspace_default_members),
            Scope::Packages(names) => package.named(names),
        })
    }

    pub(super) fn targets(&self, scope: &Scope) -> Vec<String> {
        let tests: BTreeSet<&str> = self.selected(scope).flat_map(Package::tests).collect();
        Self::UNITS
            .iter()
            .take_while(|unit| **unit != Self::TEST)
            .map(|unit| (*unit).to_owned())
            .chain(tests.into_iter().flat_map(|name| [Self::TEST.to_owned(), name.to_owned()]))
            .collect()
    }
}

impl FromStr for Workspace {
    type Err = Failure<'static>;

    fn from_str(text: &str) -> Result<Workspace, Failure<'static>> {
        serde_json::from_str(text).map_err(Failure::Metadata)
    }
}
