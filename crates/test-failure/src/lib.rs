use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use console_ports::Inherited;
use json_ports::Decode;
use toml_ports::Toml;

#[derive(Debug, PartialEq, Eq)]
pub struct Failed(String);

impl<Failure: fmt::Display> From<Failure> for Failed {
    fn from(failure: Failure) -> Failed {
        Failed(failure.to_string())
    }
}

#[derive(Debug)]
pub struct Golden(PathBuf);

impl Golden {
    const RECORD: &'static str = "RECORD_GOLDENS";

    #[must_use]
    pub fn at(manifest: &Path, relative: &str) -> Golden {
        Golden(manifest.join(relative))
    }

    pub fn checked(&self, shown: &str) -> Result<(), Failed> {
        Inherited.variable(OsStr::new(Golden::RECORD)).map_or(Ok(()), |_| fs::write(&self.0, shown))?;
        let golden = fs::read_to_string(&self.0)?;
        (golden == shown).then_some(()).ok_or_else(|| {
            Failed(format!(
                "{} differs from what was shown; rerun with {}=1 to record it:\n{shown}",
                self.0.display(),
                Golden::RECORD
            ))
        })
    }
}

#[derive(Debug)]
pub struct Cases;

impl Cases {
    pub fn read<Read: Decode>(path: &Path) -> Result<Read, Failed> {
        Ok(Toml::read(&fs::read_to_string(path)?)?)
    }
}
