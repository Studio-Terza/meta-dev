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
        OsString::from(match unit {
            Unit::Library => "--lib",
            Unit::Bins => "--bins",
            Unit::Examples => "--examples",
            Unit::Benches => "--benches",
        })
    }
}
