use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

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

    pub fn of_this_repository() -> Result<World, Box<dyn Error>> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().map(Path::to_path_buf).unwrap_or_default();
        let manifest = Self::table(&root.join(Self::MANIFEST))?;
        let patterns: Vec<String> = manifest
            .get("workspace")
            .and_then(|workspace| workspace.get("members"))
            .and_then(Value::as_array)
            .map(|members| members.iter().filter_map(Value::as_str).map(str::to_owned).collect())
            .unwrap_or_default();
        let mut members = BTreeMap::new();
        for pattern in &patterns {
            for directory in Self::directories(&root, pattern)? {
                let member = Self::member(&root, directory)?;
                members.insert(member.name.clone(), member);
            }
        }
        Ok(World { root, members })
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
            .filter(|(_, member)| {
                path.starts_with(&member.directory) || member.reads.iter().any(|read| path.starts_with(read))
            })
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
                    if member.reads.is_empty() { Json::Null } else { json!({ "reach": { "reads": member.reads } }) };
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

    fn closure(&self, mut reached: BTreeSet<String>) -> BTreeSet<String> {
        loop {
            let grown: BTreeSet<String> = self
                .members
                .iter()
                .filter(|(_, member)| !member.dependencies.is_disjoint(&reached))
                .map(|(name, _)| name.clone())
                .chain(reached.iter().cloned())
                .collect();
            if grown.len() == reached.len() {
                return reached;
            }
            reached = grown;
        }
    }

    fn directories(root: &Path, pattern: &str) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        let Some(parent) = pattern.strip_suffix(Self::ALL_BELOW) else {
            return Ok(vec![PathBuf::from(pattern)]);
        };
        let mut found = Vec::new();
        for entry in fs::read_dir(root.join(parent))? {
            let name = entry?.file_name();
            let directory = Path::new(parent).join(name);
            if root.join(&directory).join(Self::MANIFEST).is_file() {
                found.push(directory);
            }
        }
        Ok(found)
    }

    fn member(root: &Path, directory: PathBuf) -> Result<Member, Box<dyn Error>> {
        let manifest = Self::table(&root.join(&directory).join(Self::MANIFEST))?;
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

    fn table(path: &Path) -> Result<Table, Box<dyn Error>> {
        Ok(fs::read_to_string(path)?.parse::<Table>()?)
    }
}
