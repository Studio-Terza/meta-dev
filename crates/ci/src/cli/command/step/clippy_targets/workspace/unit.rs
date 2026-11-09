use std::ffi::OsString;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Unit {
    Library,
    Bins,
    Examples,
    Benches,
}

impl From<Unit> for OsString {
    fn from(unit: Unit) -> OsString {
        match unit {
            Unit::Library => OsString::from("--lib"),
            Unit::Bins => OsString::from("--bins"),
            Unit::Examples => OsString::from("--examples"),
            Unit::Benches => OsString::from("--benches"),
        }
    }
}
