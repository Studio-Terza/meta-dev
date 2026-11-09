mod boundary;
mod key_lines;
mod members;

use std::collections::BTreeSet;
use std::io;
use std::path::{Path, PathBuf};

use file_ports::{FileFailure, Files, Text};
use newtype::Newtype;
use serde_core::Deserialize;
use toml::de::{DeTable, Deserializer};
use toml::{Table, Value};

use crate::dependencies::Dependencies;
use crate::failure::Failure;
use crate::name::Name;
use crate::path_dependencies::PathDependencies;
use crate::source_text::SourceText;
use crate::untested::Untested;
use crate::word::Word;
pub use key_lines::{KeyLines, TableKey};
use members::Members;

#[derive(Debug)]
pub struct Manifest {
    directory: PathBuf,
    table: Table,
    lines: KeyLines,
    dependencies: Dependencies,
}

impl Manifest {
    const FILE: &'static str = "Cargo.toml";
    const PACKAGE: &'static str = "package";
    const WORKSPACE: &'static str = "workspace";
    const PATH: &'static str = "path";
    const LIBRARY: &'static str = "lib";
    const TEST: &'static str = "test";
    const NAME: &'static str = "name";
    const SOURCE_EXTENSION: &'static str = "rs";
    const MAIN_FILE: &'static str = "main.rs";
    const TARGET_LISTS: [&'static str; 4] = ["test", "bin", "example", "bench"];
    const TESTED_LISTS: [&'static str; 3] = ["test", "bin", "bench"];
    const TARGET_DIRECTORIES: [&'static str; 4] = ["tests", "src/bin", "examples", "benches"];
    const DEPENDENCIES: [&'static str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];

    pub fn read_many(files: &impl Files, directories: &[impl AsRef<Path>]) -> Vec<Result<Option<Manifest>, Failure>> {
        let paths: Vec<PathBuf> = directories.iter().map(|directory| directory.as_ref().join(Self::FILE)).collect();
        directories
            .iter()
            .zip(files.read_text_many(&paths))
            .map(|(directory, read)| Self::parsed(directory.as_ref(), read))
            .collect()
    }

    fn parsed(directory: &Path, read: Result<Text, FileFailure>) -> Result<Option<Manifest>, Failure> {
        match read.map(Newtype::into_inner).map_err(io::Error::from) {
            Ok(text) => Self::typed(directory, &SourceText::new(text))
                .map(Some)
                .map_err(|source| Failure::Manifest { path: directory.join(Self::FILE), source }),
            Err(source) => Some(source.kind())
                .filter(|kind| *kind == io::ErrorKind::NotFound)
                .map_or(Err(Failure::Io { path: directory.join(Self::FILE), source }), |_| Ok(None)),
        }
    }

    fn typed(directory: &Path, text: &SourceText) -> Result<Manifest, toml::de::Error> {
        let document = DeTable::parse(text.as_ref())?;
        let lines = KeyLines::read(text, document.get_ref());
        let dependencies = Dependencies::listed(text, document.get_ref());
        let table = Table::deserialize(Deserializer::from(document))?;
        Ok(Manifest { directory: directory.to_path_buf(), table, lines, dependencies })
    }

    #[must_use]
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    #[must_use]
    pub fn package(&self) -> Option<&Manifest> {
        self.table.contains_key(Self::PACKAGE).then_some(self)
    }

    #[must_use]
    pub fn package_name(&self) -> Option<Name> {
        self.table.get(Self::PACKAGE)?.get("name")?.as_str().map(Name::from)
    }

    #[must_use]
    pub fn dependencies(&self) -> &Dependencies {
        &self.dependencies
    }

    #[must_use]
    pub fn workspace(&self) -> Option<Members> {
        self.table.get(Self::WORKSPACE).map(|_| {
            Members::new(self.workspace_names(Word::new("members")), self.workspace_names(Word::new("exclude")))
        })
    }

    #[must_use]
    pub fn named_paths(&self) -> Vec<PathBuf> {
        let named = |path: &Path| Some(path.to_path_buf());
        self.path_dependencies(named)
            .into_inner()
            .into_iter()
            .chain(self.declared_roots(named))
            .chain(self.untested_targets(named).into_inner())
            .collect()
    }

    #[must_use]
    pub fn path_dependencies(&self, canonical: impl Fn(&Path) -> Option<PathBuf>) -> PathDependencies {
        let workspace = self.table.get(Self::WORKSPACE).and_then(|value| value.get("dependencies"));
        PathDependencies::new(
            Self::DEPENDENCIES
                .iter()
                .filter_map(|name| self.table.get(*name))
                .chain(workspace)
                .filter_map(Value::as_table)
                .flat_map(Table::values)
                .filter_map(|dependency| dependency.get(Self::PATH)?.as_str())
                .filter_map(|path| canonical(&self.directory.join(path)))
                .collect(),
        )
    }

    #[must_use]
    pub fn declared_roots(&self, canonical: impl Fn(&Path) -> Option<PathBuf>) -> BTreeSet<PathBuf> {
        let library = self.table.get(Self::LIBRARY).into_iter();
        let listed = Self::TARGET_LISTS.iter().filter_map(|key| self.table.get(*key)).filter_map(Value::as_array);
        library
            .chain(listed.flatten())
            .filter_map(|target| target.get(Self::PATH)?.as_str())
            .filter_map(|path| canonical(&self.directory.join(path)))
            .collect()
    }

    #[must_use]
    pub fn untested_targets(&self, canonical: impl Fn(&Path) -> Option<PathBuf>) -> Untested {
        let library = self.table.get(Self::LIBRARY).into_iter();
        let listed = Self::TESTED_LISTS.iter().filter_map(|key| self.table.get(*key)).filter_map(Value::as_array);
        Untested::new(
            library
                .chain(listed.flatten())
                .filter(|target| target.get(Self::TEST).and_then(Value::as_bool) == Some(false))
                .flat_map(|target| {
                    let explicit = target.get(Self::PATH).and_then(Value::as_str).map(|path| vec![PathBuf::from(path)]);
                    explicit.unwrap_or_else(|| {
                        Self::default_files(target.get(Self::NAME).and_then(Value::as_str).map(Name::from))
                    })
                })
                .filter_map(|path| canonical(&self.directory.join(path)))
                .collect(),
        )
    }

    fn default_files(name: Option<Name>) -> Vec<PathBuf> {
        name.into_iter()
            .flat_map(|name| {
                Self::TARGET_DIRECTORIES
                    .iter()
                    .map(move |directory| PathBuf::from(directory).join(&name))
                    .flat_map(|target| [target.with_extension(Self::SOURCE_EXTENSION), target.join(Self::MAIN_FILE)])
            })
            .collect()
    }

    fn workspace_names(&self, key: Word<'_>) -> Vec<Name> {
        self.table
            .get(Self::WORKSPACE)
            .and_then(|workspace| workspace.get(key.into_inner()))
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(Name::from)
            .collect()
    }
}

impl AsRef<KeyLines> for Manifest {
    fn as_ref(&self) -> &KeyLines {
        &self.lines
    }
}
