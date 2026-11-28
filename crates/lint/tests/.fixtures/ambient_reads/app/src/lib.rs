use std::env;
use std::env::args_os;
use std::time::Instant;

pub struct Reads;

impl Reads {
    pub fn home() -> Option<std::ffi::OsString> {
        env::var_os("HOME")
    }

    pub fn now() -> std::time::SystemTime {
        std::time::SystemTime::now()
    }

    pub fn started() -> Instant {
        Instant::now()
    }

    pub fn first() -> Option<std::ffi::OsString> {
        args_os().next()
    }

    pub fn environment() -> std::path::PathBuf {
        env::temp_dir()
    }
}
