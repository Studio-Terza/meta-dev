use std::env;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use crate::Variables;

#[derive(Clone, Copy, Debug, Default)]
pub struct Inherited;

impl Inherited {
    #[must_use]
    pub fn variable(self, name: &OsStr) -> Option<OsString> {
        env::var_os(name)
    }

    #[must_use]
    pub fn variables(self) -> Variables {
        env::vars_os().collect()
    }

    #[must_use]
    pub fn arguments(self) -> Vec<OsString> {
        env::args_os().skip(1).collect()
    }

    #[must_use]
    pub fn temporary(self) -> PathBuf {
        env::temp_dir()
    }
}
