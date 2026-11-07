use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::path::{Path, PathBuf};

use e2e::ports::System;
use file_ports::{Files, Kind};
use serde_json::{Value as Json, json};
use toml::{Table, Value};

#[derive(Debug)]
struct Member {
    name: String,
    directory: PathBuf,
    dependencies: BTreeSet<String>,
    reads: Vec<PathBuf>,
}

#[derive(Debug)]
pub struct World {
    root: PathBuf,
    members: BTreeMap<String, Member>,
}

impl World {
    const MANIFEST: &'static str = "Cargo.toml";
    const ALL_BELOW: &'static str = "/*";
    const DEPENDENCY_TABLES: [&'static str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];

    pub fn of_this_repository() -> Result<Self, Box<dyn Error>> {
        Self::of(&System::manifests()?, System::root())
    }

    fn of(files: &impl Files, root: PathBuf) -> Result<Self, Box<dyn Error>> {
        let manifest = Self::table(files, &root.join(Self::MANIFEST))?;
        let patterns: Vec<String> = manifest
            .get("workspace")
            .and_then(|workspace| workspace.get("members"))
            .and_then(Value::as_array)
            .map(|members| members.iter().filter_map(Value::as_str).map(str::to_owned).collect())
            .unwrap_or_default();
        let directories: Vec<Vec<PathBuf>> =
            patterns.iter().map(|pattern| Self::directories(files, &root, pattern)).collect::<Result<_, _>>()?;
        let members: BTreeMap<String, Member> = directories
            .into_iter()
            .flatten()
            .map(|directory| Self::member(files, &root, directory).map(|member| (member.name.clone(), member)))
            .collect::<Result<_, _>>()?;
        Ok(Self { root, members })
    }

    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.members.keys()
    }

    pub fn total(&self) -> usize {
        self.members.len()
    }

    pub fn source_of(&self, name: &str) -> PathBuf {
        self.below(name, &["src", "lib.rs"])
    }

    pub fn below(&self, name: &str, parts: &[&str]) -> PathBuf {
        let directory = self.members.get(name).map(|member| member.directory.clone()).unwrap_or_default();
        parts.iter().fold(directory, |path, part| path.join(part))
    }

    pub fn reached(&self, changed: &[PathBuf]) -> BTreeSet<String> {
        self.closure(changed.iter().flat_map(|path| self.owners(path)).collect())
    }

    pub fn owners(&self, path: &Path) -> BTreeSet<String> {
        self.members
            .iter()
            .filter(|(_, member)| member.reads.iter().chain([&member.directory]).any(|read| path.starts_with(read)))
            .map(|(name, _)| name.clone())
            .collect()
    }

    pub fn metadata(&self) -> String {
        let packages: Vec<Json> = self
            .members
            .iter()
            .map(|(name, member)| {
                let dependencies: Vec<Json> = member.dependencies.iter().map(|name| json!({ "name": name })).collect();
                let metadata =
                    member.reads.first().map_or(Json::Null, |_| json!({ "reach": { "reads": member.reads } }));
                json!({
                    "name": name,
                    "manifest_path": self.root.join(&member.directory).join(Self::MANIFEST),
                    "dependencies": dependencies,
                    "metadata": metadata,
                })
            })
            .collect();
        json!({ "workspace_root": self.root, "packages": packages }).to_string()
    }

    fn closure(&self, reached: BTreeSet<String>) -> BTreeSet<String> {
        let grown: BTreeSet<String> = self
            .members
            .iter()
            .filter(|(_, member)| member.dependencies.intersection(&reached).next().is_some())
            .map(|(name, _)| name.clone())
            .chain(reached.iter().cloned())
            .collect();
        Some(grown).filter(|grown| grown.len() != reached.len()).map_or(reached, |grown| self.closure(grown))
    }

    fn directories(files: &impl Files, root: &Path, pattern: &str) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        match pattern.strip_suffix(Self::ALL_BELOW) {
            None => Ok(vec![PathBuf::from(pattern)]),
            Some(parent) => Self::listed(files, root, parent),
        }
    }

    fn listed(files: &impl Files, root: &Path, parent: &str) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        Ok(files
            .entries(&root.join(parent))?
            .iter()
            .map(|entry| Path::new(parent).join(entry.file_name().unwrap_or_default()))
            .filter(|directory| files.kind(&root.join(directory).join(Self::MANIFEST)) == Kind::File)
            .collect())
    }

    fn member(files: &impl Files, root: &Path, directory: PathBuf) -> Result<Member, Box<dyn Error>> {
        let manifest = Self::table(files, &root.join(&directory).join(Self::MANIFEST))?;
        let package = manifest.get("package");
        let name =
            package.and_then(|package| package.get("name")).and_then(Value::as_str).unwrap_or_default().to_owned();
        let dependencies = Self::DEPENDENCY_TABLES
            .iter()
            .filter_map(|table| manifest.get(*table).and_then(Value::as_table))
            .flat_map(|table| {
                table
                    .iter()
                    .map(|(key, value)| value.get("package").and_then(Value::as_str).unwrap_or(key.as_str()).to_owned())
            })
            .collect();
        let reads = package
            .and_then(|package| package.get("metadata"))
            .and_then(|metadata| metadata.get("reach"))
            .and_then(|reach| reach.get("reads"))
            .and_then(Value::as_array)
            .map(|reads| reads.iter().filter_map(Value::as_str).map(PathBuf::from).collect())
            .unwrap_or_default();
        Ok(Member { name, directory, dependencies, reads })
    }

    fn table(files: &impl Files, path: &Path) -> Result<Table, Box<dyn Error>> {
        Ok(files.read_text(path)?.as_ref().parse::<Table>()?)
    }
}
