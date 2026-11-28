use std::fmt::Display;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::{panic, thread};

use crate::boundary::{CaseName, Drift, Golden, Story};
use crate::{Failed, Fixture};

#[derive(Debug)]
pub struct Cases<Row>(PhantomData<Row>);

#[derive(Debug)]
pub struct Case<Row> {
    name: CaseName,
    directory: PathBuf,
    row: Row,
}

impl<Row: Fixture + Send> Cases<Row> {
    pub fn drifted<Printed: Display>(
        printing: impl Fn(Case<Row>) -> Result<Printed, Failed> + Sync,
    ) -> Result<Vec<Drift>, Failed> {
        let story = Story::of::<Row>();
        let rows = story.rows::<Row>()?;
        let mut entries = story.entries()?;
        let cases = story.cases()?;
        let paired = rows
            .into_iter()
            .map(|(name, row)| {
                let entry = entries.remove(&name).unwrap_or_default();
                let golden = entry.golden(&name)?;
                let directory = entry.directory().map_or_else(|| cases.clone(), Path::to_path_buf);
                Ok((golden, Case { name, directory, row }))
            })
            .collect::<Result<Vec<(Golden, Case<Row>)>, Failed>>()?;
        let shared = &printing;
        let compared: Vec<Result<Option<Drift>, Failed>> = thread::scope(|scope| {
            let running: Vec<_> = paired
                .into_iter()
                .map(|(golden, case)| {
                    scope.spawn(move || {
                        let name = case.name.clone();
                        golden.compared(&name, shared(case)?)
                    })
                })
                .collect();
            running
                .into_iter()
                .map(|handle| match handle.join() {
                    Ok(compared) => compared,
                    Err(panicked) => panic::resume_unwind(panicked),
                })
                .collect()
        });
        compared.into_iter().filter_map(Result::transpose).collect()
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
