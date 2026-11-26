use std::fmt;
use std::path::{Path, PathBuf};

use crate::label::Law;

#[derive(Debug)]
pub struct Runner<'model> {
    laws: Vec<Law<'model>>,
}

impl<'model> Runner<'model> {
    const TESTS: &'static str = "tests";
    const GENERATED: &'static str = "generated";
    const FILE: &'static str = "laws.rs";

    pub fn file(&self) -> Option<PathBuf> {
        let root: &Path = self.home()?;
        Some(root.parent()?.parent()?.join(Self::TESTS).join(Self::GENERATED).join(Self::FILE))
    }

    pub fn home(&self) -> Option<&'model Path> {
        self.laws.first().copied().map(Law::home)
    }
}

impl<'model> From<&[Law<'model>]> for Runner<'model> {
    fn from(laws: &[Law<'model>]) -> Runner<'model> {
        Runner { laws: laws.to_vec() }
    }
}

impl fmt::Display for Runner<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "use laws::proptest::prelude::any;")?;
        self.laws.iter().try_for_each(|law| write!(formatter, "{law}"))
    }
}
