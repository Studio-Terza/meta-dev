use std::collections::BTreeSet;
use std::fmt;
use std::path::{Component, PathBuf};

use grammar_lines::{Fields, Lines};

use crate::boundary::Said;
use crate::failure::Failure;

#[derive(Debug)]
pub(super) struct Shards(usize);

#[derive(Debug)]
pub(super) struct Changed;

impl Shards {
    pub(super) fn sharing(selected: usize, per_shard: usize) -> Shards {
        Shards(selected.div_ceil(per_shard).max(1))
    }

    pub(super) fn count(&self) -> usize {
        self.0
    }
}

impl Changed {
    const TESTS: &'static str = "tests";

    pub(super) fn tested(numstat: &str) -> Result<BTreeSet<PathBuf>, Failure> {
        let tests = Component::Normal(Changed::TESTS.as_ref());
        let paths: Vec<PathBuf> = numstat
            .parse::<Lines>()
            .into_iter()
            .flatten()
            .map(|line| match line.parse::<Fields>().as_ref().map(AsRef::as_ref) {
                Ok([_, _, path]) => Ok(PathBuf::from(path)),
                Ok(_) | Err(_) => Err(Failure::Numstat(Said::spoken(&line))),
            })
            .collect::<Result<_, Failure>>()?;
        Ok(paths
            .into_iter()
            .filter(|path| path.components().any(|part| part == tests))
            .map(|path| path.components().take_while(|part| *part != tests).collect())
            .collect())
    }
}

impl fmt::Display for Shards {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indices: Vec<String> = (0..self.0).map(|index| index.to_string()).collect();
        write!(formatter, "[{}]", indices.join(","))
    }
}
