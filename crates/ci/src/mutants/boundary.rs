use std::fmt;
use std::path::PathBuf;

use grammar::{Fields, Lines};

use super::{Deletion, Row};
use crate::boundary::Said;
use crate::failure::Failure;

#[derive(Debug)]
pub(super) struct Shards(usize);

impl Shards {
    pub(super) fn sharing(untested: usize, per_shard: usize) -> Shards {
        Shards(untested.div_ceil(per_shard).max(1))
    }

    pub(super) fn count(&self) -> usize {
        self.0
    }
}

impl Row {
    const UNTOUCHED: &'static str = "0";

    pub(super) fn read(numstat: &str) -> Result<Vec<Row>, Failure> {
        numstat
            .parse::<Lines>()
            .into_iter()
            .flatten()
            .map(|line| match line.parse::<Fields>().as_ref().map(AsRef::as_ref) {
                Ok([_, deleted, path]) => Ok(Row { deleted: Deletion::counted(deleted), path: PathBuf::from(path) }),
                Ok(_) | Err(_) => Err(Failure::Numstat(Said::spoken(&line))),
            })
            .collect()
    }
}

impl Deletion {
    fn counted(deleted: &str) -> Deletion {
        if deleted == Row::UNTOUCHED { Deletion::Untouched } else { Deletion::Lost }
    }
}

impl fmt::Display for Shards {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indices: Vec<String> = (0..self.0).map(|index| index.to_string()).collect();
        write!(formatter, "[{}]", indices.join(","))
    }
}
