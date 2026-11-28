use std::env;
use std::ffi::{OsStr, OsString};

#[derive(Clone, Copy, Debug, Default)]
pub struct Inherited;

impl Inherited {
    #[must_use]
    pub fn variable(self, name: &OsStr) -> Option<OsString> {
        env::var_os(name)
    }

    #[must_use]
    pub fn arguments(self) -> Vec<OsString> {
        env::args_os().skip(1).collect()
    }
}
