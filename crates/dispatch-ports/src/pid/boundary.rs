use std::ffi::OsString;

use super::Pid;

impl Pid {
    pub(crate) fn group(self) -> OsString {
        OsString::from(format!("-{}", self.0))
    }
}
