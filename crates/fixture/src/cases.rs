use std::fmt::Display;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use crate::{CaseName, Drift, Failed, Fixture};

#[derive(Debug)]
pub struct Cases<Row>(PhantomData<Row>);

#[derive(Debug)]
pub struct Case<Row> {
    name: CaseName,
    directory: Option<PathBuf>,
    row: Option<Row>,
}

impl<Row: Fixture> Cases<Row> {
    pub fn drifted<Printed: Display>(
        printing: impl Fn(&Case<Row>) -> Result<Printed, Failed>,
    ) -> Result<Vec<Drift>, Failed> {
        let mut rows = Row::STORY.rows::<Row>()?;
        let entries = Row::STORY.entries()?;
        let compared = entries.into_iter().map(|(name, entry)| {
            let golden = entry.golden(&name)?;
            let case = Case { row: rows.remove(&name), directory: entry.directory().map(Path::to_path_buf), name };
            golden.compared(&case.name, printing(&case)?)
        });
        compared.filter_map(Result::transpose).collect()
    }
}

impl<Row> Case<Row> {
    #[must_use]
    pub fn name(&self) -> &CaseName {
        &self.name
    }

    #[must_use]
    pub fn directory(&self) -> Option<&Path> {
        self.directory.as_deref()
    }

    #[must_use]
    pub fn row(&self) -> Option<&Row> {
        self.row.as_ref()
    }
}
