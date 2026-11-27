use std::ffi::OsString;

use crate::Cli;

impl From<Vec<String>> for Cli {
    fn from(words: Vec<String>) -> Cli {
        Cli::from_args(words.into_iter().map(OsString::from))
    }
}
