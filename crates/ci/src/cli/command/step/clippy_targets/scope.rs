use std::ffi::{OsStr, OsString};

use super::name::PackageId;
use super::workspace::Package;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Scope {
    Default,
    Workspace,
    Packages(Vec<OsString>),
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
            Scope::Default => packages.iter().filter_map(|package| package.defaulted(defaults)).collect(),
            Scope::Packages(names) => packages.iter().filter_map(|package| package.named(names)).collect(),
        }
    }

    fn packages(words: &[&OsStr]) -> Vec<OsString> {
        words
            .iter()
            .zip(words.iter().skip(1))
            .filter(|(flag, _)| Self::PACKAGE.iter().any(|package| ***flag == **package))
            .map(|(_, name)| (*name).to_owned())
            .collect()
    }

    fn before_end(arguments: &[OsString]) -> Vec<&OsStr> {
        arguments.iter().map(OsString::as_os_str).take_while(|word| **word != *Self::END).collect()
    }
}

impl From<&[OsString]> for Scope {
    fn from(arguments: &[OsString]) -> Scope {
        let words = Scope::before_end(arguments);
        let packages = Scope::packages(&words);
        let workspace =
            words.iter().find(|word| Self::WORKSPACE.iter().any(|flag| ***word == **flag)).map(|_| Scope::Workspace);
        let named = packages.first().map(|_| Scope::Packages(packages.clone()));
        workspace.or(named).unwrap_or(Scope::Default)
    }
}
