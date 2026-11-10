use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::Deserialize;
use text_lines::OneLine;

use crate::cli::command::failure::Failure;

#[derive(Debug, Deserialize)]
pub(super) struct Located {
    packages: Vec<Listed>,
}

#[derive(Debug, Deserialize)]
struct Listed {
    name: OneLine,
    manifest_path: PathBuf,
}

impl Located {
    pub(super) const METADATA: [&'static str; 4] = ["metadata", "--locked", "--format-version", "1"];
    const PACKAGE: &'static str = "libsqlite3-sys";
    const SOURCE: [&'static str; 2] = ["sqlite3", "sqlite3.c"];
    const FLAGS: [&'static str; 23] = [
        "-O2",
        "-DSQLITE_CORE",
        "-DSQLITE_DEFAULT_FOREIGN_KEYS=1",
        "-DSQLITE_ENABLE_API_ARMOR",
        "-DSQLITE_ENABLE_COLUMN_METADATA",
        "-DSQLITE_ENABLE_DBSTAT_VTAB",
        "-DSQLITE_ENABLE_FTS3",
        "-DSQLITE_ENABLE_FTS3_PARENTHESIS",
        "-DSQLITE_ENABLE_FTS5",
        "-DSQLITE_ENABLE_JSON1",
        "-DSQLITE_ENABLE_LOAD_EXTENSION=1",
        "-DSQLITE_ENABLE_MEMORY_MANAGEMENT",
        "-DSQLITE_ENABLE_RTREE",
        "-DSQLITE_ENABLE_STAT4",
        "-DSQLITE_SOUNDEX",
        "-DSQLITE_THREADSAFE=1",
        "-DSQLITE_USE_URI",
        "-DHAVE_USLEEP=1",
        "-DHAVE_ISNAN",
        "-D_POSIX_THREAD_SAFE_FUNCTIONS",
        "-DSQLITE_DEFAULT_MEMSTATUS=0",
        "-DHAVE_LOCALTIME_R",
        "-c",
    ];

    pub(super) fn source(&self) -> Result<PathBuf, Failure<'static>> {
        self.packages
            .iter()
            .find(|listed| listed.name.as_ref() == Self::PACKAGE)
            .and_then(|listed| listed.manifest_path.parent())
            .map(|package| Self::SOURCE.iter().fold(package.to_path_buf(), |path, part| path.join(part)))
            .ok_or(Failure::NoSqlite)
    }

    pub(super) fn compile_arguments(source: &Path, object: &Path) -> Vec<OsString> {
        Self::FLAGS
            .iter()
            .map(OsString::from)
            .chain([source.as_os_str().to_owned(), OsString::from("-o"), object.as_os_str().to_owned()])
            .collect()
    }
}

impl FromStr for Located {
    type Err = Failure<'static>;

    fn from_str(text: &str) -> Result<Located, Failure<'static>> {
        serde_json::from_str(text).map_err(Failure::Metadata)
    }
}
