use std::ffi::{OsStr, OsString};
use std::io;

use crate::Heard;

pub trait Environment {
    fn variable(&self, name: &OsStr) -> Option<OsString>;

    fn input(&self) -> io::Result<Heard>;
}
