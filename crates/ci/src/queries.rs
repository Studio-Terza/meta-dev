use std::io::Write;
use std::path::PathBuf;

use design_messages::StreamSchema;
use port_core::Run;
use query_cache::Cache;
use sqlite_ports::SqliteOpener;
use store_ports::{Opens, Parses};

use crate::cli::Exit;
use crate::failure::Failure;

#[derive(Debug)]
pub struct Queries {
    root: PathBuf,
    mode: Mode,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Mode {
    Checked,
    Recorded,
}

impl Queries {
    const DRIFTED: &'static str =
        "::error::the .sqlx query cache drifted from the schema; rewrite it with ci queries <workspace> --record";

    pub(crate) fn new(root: PathBuf, mode: Mode) -> Queries {
        Queries { root, mode }
    }

    pub(crate) fn run(&self, out: &mut impl Write) -> Result<Exit, Failure> {
        let run = Run::root();
        let files = run.port(file_ports::Disk);
        let store = run.port(SqliteOpener.in_memory(&SqliteOpener.parsed(StreamSchema::SQL)?));
        let cache = Cache::rooted(&self.root);
        let exit = match self.mode {
            Mode::Checked => {
                let drift = cache.drifted(&files, &store)?;
                drift.settled().map_or_else(
                    || writeln!(out, "{}\n{drift}", Queries::DRIFTED).map(|()| Exit::Failed),
                    |()| Ok(Exit::Passed),
                )
            }
            Mode::Recorded => {
                let drift = cache.recorded(&files, &store)?;
                write!(out, "{drift}").map(|()| Exit::Passed)
            }
        }?;
        Ok(exit)
    }
}
