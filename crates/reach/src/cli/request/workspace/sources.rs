use std::collections::BTreeSet;
use std::fmt;
use std::path::Path;

#[derive(Debug)]
pub struct Sources<'meta> {
    directories: BTreeSet<&'meta Path>,
}

impl<'meta> Sources<'meta> {
    const MANIFEST: &'static str = "Cargo.toml";
    const SOURCE_TREE: &'static str = "src/**";

    pub fn new(directories: BTreeSet<&'meta Path>) -> Sources<'meta> {
        Sources { directories }
    }
}

impl fmt::Display for Sources<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.directories.iter().try_for_each(|directory| {
            writeln!(formatter, "{}", directory.join(Self::MANIFEST).display())?;
            writeln!(formatter, "{}", directory.join(Self::SOURCE_TREE).display())
        })
    }
}
