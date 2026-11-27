use std::ffi::{OsStr, OsString};
use std::process::ExitStatus;

use newtype::{Arbitrary, Deserialize, Display, Newtype};

use super::Ended;

#[derive(Newtype, Arbitrary, Display, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pid(u32);

#[derive(Debug)]
pub(crate) struct Group(OsString);

impl Pid {
    pub(crate) const KILL: &'static str = "kill";
    pub(crate) const SIGNAL: [&'static str; 2] = ["-KILL", "--"];

    pub(crate) fn group(self) -> Group {
        Group(OsString::from(format!("-{}", self.0)))
    }

    pub(crate) fn listed(self) -> OsString {
        OsString::from(self.0.to_string())
    }
}

impl AsRef<OsStr> for Group {
    fn as_ref(&self) -> &OsStr {
        &self.0
    }
}

impl From<ExitStatus> for Ended {
    fn from(status: ExitStatus) -> Ended {
        Some(status).filter(ExitStatus::success).map_or(Ended::Absent, |_signalled| Ended::Killed)
    }
}
