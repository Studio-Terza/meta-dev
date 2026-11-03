use std::ffi::{OsStr, OsString};

pub trait Environment {
    fn variable(&self, name: &OsStr) -> Option<OsString>;
}
