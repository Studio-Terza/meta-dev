use std::ffi::OsString;

use super::name::{PackageId, PackageName};
use super::workspace::Package;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Scope {
    Default,
    Workspace,
    Packages(Vec<PackageName>),
}

impl Scope {
    const END: &'static str = "--";
    const WORKSPACE: [&'static str; 2] = ["--workspace", "--all"];
    const PACKAGE: [&'static str; 2] = ["-p", "--package"];

    pub(super) fn among<'package>(
        &self,
        packages: &'package [Package],
        defaults: &[PackageId],
    ) -> Vec<&'package Package> {
        match self {
            Scope::Workspace => packages.iter().collect(),
            Scope::Default => packages.iter().filter(|package| defaults.contains(package.id())).collect(),
            Scope::Packages(names) => packages.iter().filter(|package| names.contains(package.name())).collect(),
        }
    }
}

impl From<&[OsString]> for Scope {
    fn from(arguments: &[OsString]) -> Scope {
        let words: Vec<&str> =
            arguments.iter().map_while(|argument| argument.to_str()).take_while(|word| *word != Self::END).collect();
        let packages: Vec<PackageName> = words
            .iter()
            .zip(words.iter().skip(1))
            .filter(|(flag, _)| Self::PACKAGE.contains(flag))
            .filter_map(|(_, name)| PackageName::try_from(*name).ok())
            .collect();
        let workspace = words.iter().find(|word| Self::WORKSPACE.contains(word)).map(|_| Scope::Workspace);
        let named = packages.first().map(|_| Scope::Packages(packages.clone()));
        workspace.or(named).unwrap_or(Scope::Default)
    }
}
