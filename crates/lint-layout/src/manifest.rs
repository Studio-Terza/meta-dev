mod members;

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Path, PathBuf};

use file_ports::{FileFailure, Files, Text};
use newtype::Newtype;
use toml::Spanned;
use toml::de::DeTable;
use toml::{Table, Value};

use crate::failure::Failure;
use crate::line_number::LineNumber;
use crate::name::Name;
use crate::source_text::SourceText;
use crate::untested::Untested;
use crate::word::Word;
use members::Members;

#[derive(Debug)]
pub struct Manifest {
    directory: PathBuf,
    text: SourceText,
    table: Table,
}

impl Manifest {
    const FILE: &'static str = "Cargo.toml";
    const PACKAGE: &'static str = "package";
    const WORKSPACE: &'static str = "workspace";
    const PATH: &'static str = "path";
    const FEATURES: &'static str = "features";
    const LIBRARY: &'static str = "lib";
    const TEST: &'static str = "test";
    const NAME: &'static str = "name";
    const SOURCE_EXTENSION: &'static str = "rs";
    const MAIN_FILE: &'static str = "main.rs";
    const TARGET_LISTS: [&'static str; 4] = ["test", "bin", "example", "bench"];
    const TESTED_LISTS: [&'static str; 3] = ["test", "bin", "bench"];
    const TARGET_DIRECTORIES: [&'static str; 4] = ["tests", "src/bin", "examples", "benches"];
    const DEPENDENCIES: [&'static str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];

    pub fn read(files: &impl Files, directory: &Path) -> Result<Option<Manifest>, Failure> {
        Self::parsed(directory, files.read_text(&directory.join(Self::FILE)))
    }

    pub fn read_many(files: &impl Files, directories: &[PathBuf]) -> Vec<Result<Option<Manifest>, Failure>> {
        let paths: Vec<PathBuf> = directories.iter().map(|directory| directory.join(Self::FILE)).collect();
        directories
            .iter()
            .zip(files.read_text_many(&paths))
            .map(|(directory, read)| Self::parsed(directory, read))
            .collect()
    }

    fn parsed(directory: &Path, read: Result<Text, FileFailure>) -> Result<Option<Manifest>, Failure> {
        let file = directory.join(Self::FILE);
        match read.map(Newtype::into_inner).map_err(io::Error::from) {
            Ok(text) => text
                .parse::<Table>()
                .map(|table| Some(Manifest { directory: directory.to_path_buf(), text: SourceText::new(text), table }))
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
    pub fn package(&self) -> Option<&Manifest> {
        self.table.contains_key(Self::PACKAGE).then_some(self)
    }

    #[must_use]
    pub fn package_name(&self) -> Option<Name> {
        self.table.get(Self::PACKAGE)?.get("name")?.as_str().map(Name::from)
    }

    #[must_use]
    pub fn features(&self) -> BTreeMap<Name, LineNumber> {
        DeTable::parse(self.text.as_ref())
            .map(|document| {
                document
                    .get_ref()
                    .iter()
                    .filter(|(key, _)| key.get_ref() == Self::FEATURES)
                    .filter_map(|(_, value)| value.get_ref().as_table())
                    .flat_map(|features| features.keys())
                    .map(|key| (Name::from(key.get_ref().as_ref()), self.line_of(key)))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn line_of<Key>(&self, key: &Spanned<Key>) -> LineNumber {
        LineNumber::from(self.text.as_ref().get(..key.span().start).unwrap_or_default().matches('\n').count() + 1)
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
            .into_iter()
            .chain(self.declared_roots(named))
            .chain(self.untested_targets(named).into_inner())
            .collect()
    }

    #[must_use]
    pub fn path_dependencies(&self, canonical: impl Fn(&Path) -> Option<PathBuf>) -> Vec<PathBuf> {
        let workspace = self.table.get(Self::WORKSPACE).and_then(|value| value.get("dependencies"));
        Self::DEPENDENCIES
            .iter()
            .filter_map(|name| self.table.get(*name))
            .chain(workspace)
            .filter_map(Value::as_table)
            .flat_map(Table::values)
            .filter_map(|dependency| dependency.get(Self::PATH)?.as_str())
            .filter_map(|path| canonical(&self.directory.join(path)))
            .collect()
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
