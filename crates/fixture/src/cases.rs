use std::fmt::Display;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use crate::{CaseName, Drift, Failed, Fixture};

#[derive(Debug)]
pub struct Cases<Row>(PhantomData<Row>);

#[derive(Debug)]
pub struct Case<Row> {
    name: CaseName,
    directory: PathBuf,
    row: Row,
}

impl<Row: Fixture> Cases<Row> {
    pub fn drifted<Printed: Display>(
        printing: impl Fn(Case<Row>) -> Result<Printed, Failed>,
    ) -> Result<Vec<Drift>, Failed> {
        let rows = Row::STORY.rows::<Row>()?;
        let mut entries = Row::STORY.entries()?;
        let cases = Row::STORY.cases()?;
        let compared = rows.into_iter().map(|(name, row)| {
            let entry = entries.remove(&name).unwrap_or_default();
            let golden = entry.golden(&name)?;
            let directory = entry.directory().map_or_else(|| cases.clone(), Path::to_path_buf);
            let case = Case { row, directory, name: name.clone() };
            golden.compared(&name, printing(case)?)
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
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    #[must_use]
    pub fn row(&self) -> &Row {
        &self.row
    }

    #[must_use]
    pub fn into_row(self) -> Row {
        self.row
    }
}
