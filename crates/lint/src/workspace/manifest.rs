use std::collections::{BTreeMap, BTreeSet};
use std::iter;
use std::path::{Component, Path, PathBuf};

use toml_ports::{Table, Value};

use crate::tree::Tree;

#[derive(Debug, Default)]
struct Members {
    listed: Vec<String>,
    excluded: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct Manifest {
    directory: PathBuf,
    packaged: Option<()>,
    members: Option<Members>,
    depends: Vec<PathBuf>,
}

impl Members {
    const ANY: char = '*';

    fn of(workspace: Value<'_, '_>) -> Members {
        let names = |key: &str| -> Vec<String> {
            let listed = workspace.table().and_then(|table| table.get(key)).and_then(Value::array);
            listed.map_or_default(|names| names.into_iter().filter_map(Value::text).map(str::to_owned).collect())
        };
        Members { listed: names("members"), excluded: names("exclude") }
    }

    fn admits(&self, relative: &Path) -> Option<()> {
        let matched =
            |patterns: &[String]| patterns.iter().any(|pattern| Members::matches(pattern, relative).is_some());
        (matched(&self.listed) && !matched(&self.excluded)).then_some(())
    }

    fn matches(pattern: &str, relative: &Path) -> Option<()> {
        let wanted: Vec<Component<'_>> =
            Path::new(pattern).components().filter(|part| matches!(part, Component::Normal(_))).collect();
        let found: Vec<Component<'_>> = relative.components().collect();
        let fitting = wanted.iter().zip(&found).all(|(expected, part)| {
            let letters: Vec<char> = expected.as_os_str().to_string_lossy().chars().collect();
            let spelled: Vec<char> = part.as_os_str().to_string_lossy().chars().collect();
            Members::fits(&letters, &spelled).is_some()
        });
        (wanted.len() == found.len() && fitting).then_some(())
    }

    fn fits(pattern: &[char], text: &[char]) -> Option<()> {
        match pattern.split_first() {
            None => text.is_empty().then_some(()),
            Some((&Members::ANY, rest)) => Members::fits(rest, text).or_else(|| Members::fits(pattern, text.get(1..)?)),
            Some((letter, rest)) => {
                let (first, tail) = text.split_first()?;
                (first == letter).then_some(()).and_then(|()| Members::fits(rest, tail))
            }
        }
    }
}

impl Manifest {
    const DEPENDENCIES: [&'static str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];

    pub(crate) fn new(directory: PathBuf, table: Table<'_, '_>) -> Manifest {
        let workspace = table.get("workspace");
        let shared = workspace.and_then(Value::table).and_then(|inner| inner.get("dependencies"));
        let sections = Manifest::DEPENDENCIES.iter().filter_map(|name| table.get(name)).chain(shared);
        let depends = sections
            .filter_map(Value::table)
            .flat_map(Table::values)
            .filter_map(|dependency| dependency.table().and_then(|inner| inner.get("path")))
            .filter_map(Value::text)
            .map(|path| Tree::lexical(&directory.join(path)))
            .collect();
        Manifest { packaged: table.get("package").map(|_| ()), members: workspace.map(Members::of), depends, directory }
    }

    pub(crate) fn admitted(manifests: &[Manifest]) -> BTreeSet<PathBuf> {
        let by_directory: BTreeMap<&Path, &Manifest> =
            manifests.iter().map(|manifest| (manifest.directory.as_path(), manifest)).collect();
        let grown: BTreeMap<&Path, BTreeSet<PathBuf>> = manifests
            .iter()
            .filter(|manifest| manifest.members.is_some())
            .map(|root| (root.directory.as_path(), root.grown(&by_directory)))
            .collect();
        let kept = manifests.iter().filter(|manifest| manifest.packaged.is_some()).filter(|package| {
            let enclosing = match package.members {
                Some(_) => None,
                None => package.directory.ancestors().skip(1).find_map(|ancestor| grown.get(ancestor)),
            };
            enclosing.is_none_or(|members| members.contains(&package.directory))
        });
        kept.map(|package| package.directory.clone()).collect()
    }

    fn grown(&self, by_directory: &BTreeMap<&Path, &Manifest>) -> BTreeSet<PathBuf> {
        let listed: BTreeSet<PathBuf> = by_directory
            .values()
            .filter(|manifest| manifest.packaged.is_some())
            .filter(|manifest| {
                let relative = manifest.directory.strip_prefix(&self.directory);
                relative.is_ok_and(|inside| {
                    inside.as_os_str().is_empty()
                        || self.members.as_ref().and_then(|members| members.admits(inside)).is_some()
                })
            })
            .map(|manifest| manifest.directory.clone())
            .collect();
        let packaged = |path: &PathBuf| by_directory.get(path.as_path()).and_then(|manifest| manifest.packaged);
        let widened = iter::successors(Some(listed), |known| {
            let users = known.iter().filter_map(|path| by_directory.get(path.as_path())).chain([&self]);
            let depended: BTreeSet<PathBuf> = users
                .flat_map(|manifest| manifest.depends.iter())
                .filter(|path| packaged(path).is_some())
                .cloned()
                .collect();
            let reached: BTreeSet<PathBuf> = depended.difference(known).cloned().collect();
            reached.first().map(|_| known.union(&reached).cloned().collect())
        });
        widened.last().unwrap_or_default()
    }
}
