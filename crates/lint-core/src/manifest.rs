mod members;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use toml::{Table, Value};

use crate::failure::Failure;
use crate::name::Name;
use members::Members;

#[derive(Debug)]
pub struct Manifest {
    directory: PathBuf,
    table: Table,
}

impl Manifest {
    const FILE: &'static str = "Cargo.toml";
    const PACKAGE: &'static str = "package";
    const WORKSPACE: &'static str = "workspace";
    const PATH: &'static str = "path";
    const DEPENDENCIES: [&'static str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];

    pub fn read(directory: &Path) -> Result<Option<Manifest>, Failure> {
        let file = directory.join(Self::FILE);
        match fs::read_to_string(&file) {
            Ok(text) => text
                .parse::<Table>()
                .map(|table| Some(Manifest { directory: directory.to_path_buf(), table }))
                .map_err(|source| Failure::Manifest { path: file, source }),
            Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(source) => Err(Failure::Io { path: file, source }),
        }
    }

    #[must_use]
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    #[must_use]
    pub fn is_package(&self) -> bool {
        self.table.contains_key(Self::PACKAGE)
    }

    #[must_use]
    pub fn package_name(&self) -> Option<Name> {
        self.table.get(Self::PACKAGE)?.get("name")?.as_str().map(Name::from)
    }

    #[must_use]
    pub fn workspace(&self) -> Option<Members> {
        self.table
            .get(Self::WORKSPACE)
            .map(|_| Members::new(self.workspace_strings("members"), self.workspace_strings("exclude")))
    }

    #[must_use]
    pub fn path_dependencies(&self) -> Vec<PathBuf> {
        let workspace = self.table.get(Self::WORKSPACE).and_then(|value| value.get("dependencies"));
        Self::DEPENDENCIES
            .iter()
            .filter_map(|name| self.table.get(*name))
            .chain(workspace)
            .filter_map(Value::as_table)
            .flat_map(Table::values)
            .filter_map(|dependency| dependency.get(Self::PATH)?.as_str())
            .filter_map(|path| fs::canonicalize(self.directory.join(path)).ok())
            .collect()
    }

    fn workspace_strings(&self, key: &str) -> Vec<String> {
        self.table
            .get(Self::WORKSPACE)
            .and_then(|workspace| workspace.get(key))
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect()
    }
}
