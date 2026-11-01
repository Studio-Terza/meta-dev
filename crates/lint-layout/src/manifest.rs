mod members;

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use toml::{Table, Value};

use crate::failure::Failure;
use crate::name::Name;
use crate::untested::Untested;
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
    const FEATURES: &'static str = "features";
    const LIBRARY: &'static str = "lib";
    const BINARIES: &'static str = "bin";
    const TEST: &'static str = "test";
    const NAME: &'static str = "name";
    const SOURCE_EXTENSION: &'static str = "rs";
    const MAIN_FILE: &'static str = "main.rs";
    const TARGET_LISTS: [&'static str; 4] = ["test", "bin", "example", "bench"];
    const TARGET_DIRECTORIES: [&'static str; 4] = ["tests", "src/bin", "examples", "benches"];
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
    pub fn feature_names(&self) -> Vec<Name> {
        self.table
            .get(Self::FEATURES)
            .and_then(Value::as_table)
            .into_iter()
            .flat_map(Table::keys)
            .map(|name| Name::from(name.as_str()))
            .collect()
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

    #[must_use]
    pub fn declared_roots(&self) -> BTreeSet<PathBuf> {
        let library = self.table.get(Self::LIBRARY).into_iter();
        let binaries = self.table.get(Self::BINARIES).and_then(Value::as_array).into_iter().flatten();
        library
            .chain(binaries)
            .filter_map(|target| target.get(Self::PATH)?.as_str())
            .filter_map(|path| fs::canonicalize(self.directory.join(path)).ok())
            .collect()
    }

    #[must_use]
    pub fn untested_targets(&self) -> Untested {
        let library = self.table.get(Self::LIBRARY).into_iter();
        let listed = Self::TARGET_LISTS.iter().filter_map(|key| self.table.get(*key)).filter_map(Value::as_array);
        Untested::new(
            library
                .chain(listed.flatten())
                .filter(|target| target.get(Self::TEST).and_then(Value::as_bool) == Some(false))
                .flat_map(|target| {
                    let explicit = target.get(Self::PATH).and_then(Value::as_str).map(|path| vec![PathBuf::from(path)]);
                    explicit.unwrap_or_else(|| Self::default_files(target.get(Self::NAME).and_then(Value::as_str)))
                })
                .filter_map(|path| fs::canonicalize(self.directory.join(path)).ok())
                .collect(),
        )
    }

    fn default_files(name: Option<&str>) -> Vec<PathBuf> {
        name.into_iter()
            .flat_map(|name| {
                Self::TARGET_DIRECTORIES.iter().flat_map(move |directory| {
                    let target = PathBuf::from(directory).join(name);
                    [target.with_extension(Self::SOURCE_EXTENSION), target.join(Self::MAIN_FILE)]
                })
            })
            .collect()
    }

    fn workspace_strings(&self, key: &str) -> Vec<Name> {
        self.table
            .get(Self::WORKSPACE)
            .and_then(|workspace| workspace.get(key))
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(Name::from)
            .collect()
    }
}
