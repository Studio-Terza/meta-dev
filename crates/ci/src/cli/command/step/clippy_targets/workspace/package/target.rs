use serde::Deserialize;

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
    fn unit(self) -> Option<Unit> {
        match self {
            Kind::Lib | Kind::Rlib | Kind::Dylib | Kind::Cdylib | Kind::Staticlib | Kind::ProcMacro => {
                Some(Unit::Library)
            }
            Kind::Bin => Some(Unit::Bins),
            Kind::Example => Some(Unit::Examples),
            Kind::Bench => Some(Unit::Benches),
            Kind::Other => None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct Target {
    kind: Vec<Kind>,
}

impl Target {
    pub(super) fn unit(&self) -> Option<Unit> {
        self.kind.iter().find_map(|kind| kind.unit())
    }
}
