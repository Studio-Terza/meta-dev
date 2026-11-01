use std::collections::BTreeSet;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Sources {
    directories: BTreeSet<PathBuf>,
}

impl Sources {
    const MANIFEST: &'static str = "Cargo.toml";
    const SOURCE_TREE: &'static str = "src/**";

    pub fn new(directories: BTreeSet<PathBuf>) -> Sources {
        Sources { directories }
    }
}

impl fmt::Display for Sources {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.directories.iter().try_for_each(|directory| {
            writeln!(formatter, "{}", directory.join(Self::MANIFEST).display())?;
            writeln!(formatter, "{}", directory.join(Self::SOURCE_TREE).display())
        })
    }
}
