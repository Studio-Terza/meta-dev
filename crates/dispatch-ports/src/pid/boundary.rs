use std::ffi::{OsStr, OsString};
use std::process::ExitStatus;

use super::{Ended, Pid};

#[derive(Debug)]
pub(crate) struct Group(OsString);

impl AsRef<OsStr> for Group {
    fn as_ref(&self) -> &OsStr {
        &self.0
    }
}

impl Pid {
    pub(crate) fn group(self) -> Group {
        Group(OsString::from(format!("-{}", self.0)))
    }

    pub(crate) fn listed(self) -> OsString {
        OsString::from(self.0.to_string())
    }
}

impl From<ExitStatus> for Ended {
    fn from(status: ExitStatus) -> Ended {
        Some(status).filter(ExitStatus::success).map_or(Ended::Absent, |_signalled| Ended::Killed)
    }
}
