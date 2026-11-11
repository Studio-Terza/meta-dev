use std::ffi::OsString;

use grammar_cargo_option::{CargoOption, Options};

use super::name::PackageId;
use super::workspace::Package;
use crate::cli::command::failure::Failure;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Scope {
    Default,
    Workspace(Vec<OsString>),
    Named(Vec<OsString>),
}

impl Scope {
    const END: &'static str = "--";
    const WORKSPACE: [&'static str; 2] = ["workspace", "all"];
    const REFUSED: [&'static str; 12] = [
        "exclude",
        "manifest-path",
        "lib",
        "bin",
        "bins",
        "example",
        "examples",
        "test",
        "tests",
        "bench",
        "benches",
        "all-targets",
    ];

    pub(super) fn among<'package>(
        &self,
        packages: &'package [Package],
        defaults: &[PackageId],
    ) -> Result<Vec<&'package Package>, Failure<'static>> {
        match self {
            Scope::Default => Ok(packages.iter().filter_map(|package| package.defaulted(defaults)).collect()),
            Scope::Workspace(specs) => Self::resolved(specs, packages).map(|_named| packages.iter().collect()),
            Scope::Named(specs) => Self::resolved(specs, packages),
        }
    }

    fn resolved<'package>(
        specs: &[OsString],
        packages: &'package [Package],
    ) -> Result<Vec<&'package Package>, Failure<'static>> {
        specs.iter().map(|spec| Package::resolved(packages, spec)).collect()
    }

    fn named(specs: Vec<OsString>) -> Scope {
        specs.first().map(|_first| ()).map_or(Scope::Default, |()| Scope::Named(specs))
    }

    fn refused(words: &[&OsString], read: &[CargoOption<'_>]) -> Result<(), Failure<'static>> {
        words
            .iter()
            .zip(read)
            .find(|(_word, option)| option.long().is_some_and(|name| Self::REFUSED.contains(&name)))
            .map_or(Ok(()), |(word, _option)| Err(Failure::Refused((*word).clone())))
    }

    fn specs(words: &[&OsString], read: &[CargoOption<'_>]) -> Result<Vec<OsString>, Failure<'static>> {
        let following = words.iter().skip(1).map(|word| word.to_str()).chain([None]);
        read.iter()
            .zip(following)
            .filter_map(|(option, next)| option.spec(next))
            .map(|spec| spec.map(OsString::from).map_err(|_unspelled| Failure::Usage))
            .collect()
    }
}

impl TryFrom<&[OsString]> for Scope {
    type Error = Failure<'static>;

    fn try_from(arguments: &[OsString]) -> Result<Scope, Failure<'static>> {
        let words: Vec<&OsString> = arguments.iter().take_while(|word| *word != Scope::END).collect();
        let read: Vec<CargoOption<'_>> =
            words.iter().map(|word| Options::spelled(word.to_str().unwrap_or_default())).collect();
        Scope::refused(&words, &read)?;
        let whole = read.iter().filter_map(|option| option.long()).find(|name| Scope::WORKSPACE.contains(name));
        let scoped: fn(Vec<OsString>) -> Scope = whole.map_or(Scope::named, |_whole| Scope::Workspace);
        Scope::specs(&words, &read).map(scoped)
    }
}
