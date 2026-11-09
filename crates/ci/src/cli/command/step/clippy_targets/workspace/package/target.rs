use serde::Deserialize;

use super::super::super::name::TargetName;
use super::super::unit::Unit;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum Kind {
    Lib,
    Rlib,
    Dylib,
    Cdylib,
    Staticlib,
    ProcMacro,
    Bin,
    Example,
    Bench,
    #[serde(other)]
    Other,
}

impl Kind {
    fn unit(self, name: &TargetName) -> Option<Unit<'_>> {
        match self {
            Kind::Lib | Kind::Rlib | Kind::Dylib | Kind::Cdylib | Kind::Staticlib | Kind::ProcMacro => {
                Some(Unit::Library)
            }
            Kind::Bin => Some(Unit::Bins),
            Kind::Example => Some(Unit::Example(name)),
            Kind::Bench => Some(Unit::Bench(name)),
            Kind::Other => None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct Target {
    name: TargetName,
    kind: Vec<Kind>,
}

impl Target {
    pub(super) fn unit(&self) -> Option<Unit<'_>> {
        self.kind.iter().find_map(|kind| kind.unit(&self.name))
    }
}
