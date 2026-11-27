mod boundary;

use std::collections::{BTreeMap, BTreeSet};
use std::iter;
use std::path::{Component, Path, PathBuf};

use toml::de::{DeTable, DeValue};

use boundary::{Glob, Pattern};

#[derive(Debug, Default)]
struct Members {
    listed: Vec<Pattern>,
    excluded: Vec<Pattern>,
}

#[derive(Debug)]
pub(crate) struct Manifest {
    directory: PathBuf,
    packaged: Option<()>,
    members: Option<Members>,
    depends: Vec<PathBuf>,
}

impl Members {
    const MEMBERS: &'static str = "members";
    const EXCLUDE: &'static str = "exclude";

    fn of(workspace: &DeValue<'_>) -> Members {
        let names = |key: &str| -> Vec<Pattern> {
            let listed =
                workspace.as_table().and_then(|table| table.get(key)).and_then(|value| value.get_ref().as_array());
            listed.map_or_default(|names| {
                names.iter().filter_map(|name| name.get_ref().as_str()).map(|name| Pattern(name.to_owned())).collect()
            })
        };
        Members { listed: names(Members::MEMBERS), excluded: names(Members::EXCLUDE) }
    }

    fn admits(&self, relative: &Path) -> Option<()> {
        let matched = |patterns: &[Pattern]| patterns.iter().any(|pattern| pattern.matches(relative).is_some());
        (matched(&self.listed) && !matched(&self.excluded)).then_some(())
    }
}

impl Pattern {
    fn matches(&self, relative: &Path) -> Option<()> {
        let wanted: Vec<Component<'_>> =
            Path::new(&self.0).components().filter(|part| matches!(part, Component::Normal(_))).collect();
        let found: Vec<Component<'_>> = relative.components().collect();
        let fitting = wanted.iter().zip(&found).all(|(expected, part)| {
            let letters: Vec<char> = expected.as_os_str().to_string_lossy().chars().collect();
            let spelled: Vec<char> = part.as_os_str().to_string_lossy().chars().collect();
            Glob::fits(&letters, &spelled).is_some()
        });
        (wanted.len() == found.len() && fitting).then_some(())
    }
}

impl Manifest {
    const DEPENDENCIES: [&'static str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];

    pub(crate) fn new(directory: PathBuf, table: &DeTable<'_>) -> Manifest {
        let workspace = table.get("workspace").map(toml::Spanned::get_ref);
        let shared = workspace.and_then(|value| value.as_table()).and_then(|inner| inner.get("dependencies"));
        let sections = Manifest::DEPENDENCIES.iter().filter_map(|name| table.get(*name)).chain(shared);
        let depends = sections
            .filter_map(|section| section.get_ref().as_table())
            .flat_map(|dependencies| dependencies.iter().map(|(_, dependency)| dependency.get_ref()))
            .filter_map(|dependency| dependency.as_table().and_then(|inner| inner.get("path")))
            .filter_map(|path| path.get_ref().as_str())
            .map(|path| Manifest::lexical(&directory.join(path)))
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
            let reached: BTreeSet<PathBuf> = users
                .flat_map(|manifest| manifest.depends.iter())
                .filter(|path| packaged(path).is_some() && !known.contains(*path))
                .cloned()
                .collect();
            reached.first().map(|_| known.union(&reached).cloned().collect())
        });
        widened.last().unwrap_or_default()
    }

    fn lexical(path: &Path) -> PathBuf {
        path.components().fold(PathBuf::new(), |folded, component| match component {
            Component::ParentDir => folded.parent().map_or_else(|| folded.join(component), Path::to_path_buf),
            Component::CurDir => folded,
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => folded.join(component),
        })
    }
}
