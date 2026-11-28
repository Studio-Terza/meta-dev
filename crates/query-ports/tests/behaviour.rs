use std::path::PathBuf;

use design_messages::StreamSchema;
use file_ports::Disk;
use fixture::{Cases, Failed, Fixture};
use port_core::Run;
use query_ports::Cache;
use serde::Deserialize;
use sqlite_ports::SqliteOpener;
use store_ports::{Opens, Parses};

#[derive(Fixture, Deserialize, Debug)]
struct Workspace {
    root: PathBuf,
}

#[test]
fn the_workspace_cache_holds_what_the_schema_describes_of_every_query_in_its_sources() -> Result<(), Failed> {
    let drifted = Cases::<Workspace>::drifted(|case| {
        let run = Run::root();
        let store = run.port(SqliteOpener.in_memory(&SqliteOpener.parsed(StreamSchema::SQL)?));
        Ok(Cache::rooted(&case.row().root).drifted(&run.port(Disk), &store)?.to_string())
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}
