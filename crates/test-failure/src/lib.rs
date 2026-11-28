use std::env;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

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

#[derive(Debug)]
pub struct Cases;

#[derive(Debug)]
struct Machine {
    recording: Option<OsString>,
    roots: Vec<PathBuf>,
}

impl Machine {
    const RECORD: &'static str = "RECORD_GOLDENS";
    const HOME: &'static str = "HOME";

    fn read() -> &'static Machine {
        static MACHINE: OnceLock<Machine> = OnceLock::new();
        MACHINE.get_or_init(|| {
            let home = env::var_os(Machine::HOME).map(PathBuf::from);
            let roots = home.into_iter().chain([env::temp_dir()]).map(|root| root.components().collect());
            Machine {
                recording: env::var_os(Machine::RECORD),
                roots: roots.filter(|root: &PathBuf| root.parent().is_some()).collect(),
            }
        })
    }

    fn unleaked(&self, named: &Path, shown: &str) -> Result<(), Failed> {
        let leaked = self.roots.iter().find(|root| shown.contains(root.to_string_lossy().as_ref()));
        leaked.map_or(Ok(()), |root| {
            Err(Failed(format!(
                "{} shows {}, a directory of the machine running the test; show paths relative to the test's own root",
                named.display(),
                root.display()
            )))
        })
    }
}

impl Golden {
    #[must_use]
    pub fn at(manifest: &Path, relative: &str) -> Golden {
        Golden(manifest.join(relative))
    }

    pub fn checked(&self, shown: &str) -> Result<(), Failed> {
        let machine = Machine::read();
        machine.unleaked(&self.0, shown)?;
        machine.recording.as_ref().map_or(Ok(()), |_| fs::write(&self.0, shown))?;
        let golden = fs::read_to_string(&self.0)?;
        (golden == shown).then_some(()).ok_or_else(|| {
            Failed(format!(
                "{} differs from what was shown; rerun with {}=1 to record it:\n{shown}",
                self.0.display(),
                Machine::RECORD
            ))
        })
    }
}

impl Cases {
    pub fn read<Read: Decode>(path: &Path) -> Result<Read, Failed> {
        let text = fs::read_to_string(path)?;
        Machine::read().unleaked(path, &text)?;
        Ok(Toml::read(&text)?)
    }
}
