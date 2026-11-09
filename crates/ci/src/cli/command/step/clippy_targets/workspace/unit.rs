use std::ffi::OsString;

use super::super::name::TargetName;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Unit<'target> {
    Library,
    Bins,
    Example(&'target TargetName),
    Bench(&'target TargetName),
}

impl Unit<'_> {
    pub(super) fn flags(self) -> Vec<OsString> {
        match self {
            Unit::Library => vec![OsString::from("--lib")],
            Unit::Bins => vec![OsString::from("--bins")],
            Unit::Example(name) => vec![OsString::from("--example"), OsString::from(name.as_ref())],
            Unit::Bench(name) => vec![OsString::from("--bench"), OsString::from(name.as_ref())],
        }
    }
}
