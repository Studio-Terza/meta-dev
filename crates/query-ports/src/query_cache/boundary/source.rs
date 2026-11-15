use std::path::Path;

use file_ports::Disk;
use graph_reader::Reader;
use lint_reader::SourceFiles;
use lint_store_queries::{Statement, StoreQueries};

use super::Uncached;

#[derive(Debug)]
pub(super) struct Sources;

impl Sources {
    pub(super) fn queries(root: &Path) -> Result<Vec<Statement>, Uncached> {
        let readout = SourceFiles::read(&Disk, root).map_err(Uncached::Unread)?;
        Ok(StoreQueries::sql(&Reader::read_files(readout.files())))
    }
}
