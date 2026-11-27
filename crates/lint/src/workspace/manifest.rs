use std::collections::{BTreeMap, BTreeSet};
use std::iter;
use std::path::{Component, Path, PathBuf};

use toml::de::{DeTable, DeValue};

use crate::terms::Name;

mod boundary;

#[derive(Debug, Default)]
struct Members {
    listed: Vec<PathBuf>,
    excluded: Vec<PathBuf>,
}

#[derive(Debug)]
pub(crate) struct Manifest {
    directory: PathBuf,
    packaged: Option<()>,
    members: Option<Members>,
    depends: Vec<PathBuf>,
    shipping: Vec<PathBuf>,
    features: BTreeMap<Name, Vec<Enabling>>,
    requests: Vec<Enabled>,
    named: BTreeMap<Name, PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Enabled {
    package: PathBuf,
    feature: Name,
}

#[derive(Clone, Debug)]
enum Enabling {
    Own(Name),
    Dependency { dependency: Name, feature: Name },
    Optional,
}

impl Members {
    fn of(workspace: &DeValue<'_>) -> Members {
        let names = |key| -> Vec<PathBuf> {
            let listed =
                workspace.as_table().and_then(|table| table.get(key)).and_then(|value| value.get_ref().as_array());
            listed.map_or_default(|names| {
                names.iter().filter_map(|name| name.get_ref().as_str()).map(PathBuf::from).collect()
            })
        };
        Members { listed: names("members"), excluded: names("exclude") }
    }

    fn admits(&self, relative: &Path) -> Option<()> {
        let matched =
            |patterns: &[PathBuf]| patterns.iter().any(|pattern| Members::matches(pattern, relative).is_some());
        (matched(&self.listed) && !matched(&self.excluded)).then_some(())
    }

    fn matches(pattern: &Path, relative: &Path) -> Option<()> {
        let wanted: Vec<Component<'_>> =
            pattern.components().filter(|part| matches!(part, Component::Normal(_))).collect();
        let found: Vec<Component<'_>> = relative.components().collect();
        let fitting = wanted.iter().zip(&found).all(|(expected, part)| Members::fitting(expected, part).is_some());
        (wanted.len() == found.len() && fitting).then_some(())
    }
}

impl Manifest {
    const DEPENDENCIES: [&'static str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];
    const SHIPPING: [&'static str; 2] = ["dependencies", "build-dependencies"];

    pub(crate) fn new(directory: PathBuf, table: &DeTable<'_>) -> Manifest {
        let workspace = table.get("workspace").map(toml::Spanned::get_ref);
        let shared = workspace.and_then(|value| value.as_table()).and_then(|inner| inner.get("dependencies"));
        let sections = Manifest::DEPENDENCIES.iter().filter_map(|name| table.get(*name)).chain(shared);
        let depends = Manifest::pathed(&directory, sections);
        let shipping = Manifest::pathed(&directory, Manifest::SHIPPING.iter().filter_map(|name| table.get(*name)));
        let all = Manifest::DEPENDENCIES.iter().filter_map(|name| table.get(*name));
        let requests = Manifest::requested(&directory, Manifest::SHIPPING.iter().filter_map(|name| table.get(*name)));
        Manifest {
            packaged: table.get("package").map(|_| ()),
            members: workspace.map(Members::of),
            depends,
            shipping,
            features: Manifest::declared_features(table.get(Manifest::FEATURES)),
            requests,
            named: Manifest::named(&directory, all),
            directory,
        }
    }

    pub(crate) fn test_features(
        manifests: &[Manifest],
        testing: &BTreeSet<PathBuf>,
    ) -> BTreeMap<PathBuf, BTreeSet<Name>> {
        let by_directory: BTreeMap<&Path, &Manifest> =
            manifests.iter().map(|manifest| (manifest.directory.as_path(), manifest)).collect();
        let shipped =
            manifests.iter().filter(|manifest| manifest.packaged.is_some() && !testing.contains(&manifest.directory));
        let seeds: BTreeSet<Enabled> = shipped
            .flat_map(|manifest| {
                let own = Enabled { package: manifest.directory.clone(), feature: Name::spelled(Manifest::DEFAULT) };
                let defaults = manifest.requests.iter().map(|request| Enabled {
                    package: request.package.clone(),
                    feature: Name::spelled(Manifest::DEFAULT),
                });
                iter::once(own).chain(defaults).chain(manifest.requests.iter().cloned()).collect::<Vec<Enabled>>()
            })
            .collect();
        let enabled = iter::successors(Some(seeds), |known| {
            let reached: BTreeSet<Enabled> = known
                .iter()
                .flat_map(|enabled| Manifest::implied(&by_directory, enabled))
                .filter(|enabled| !known.contains(enabled))
                .collect();
            reached.first().map(|_| known.union(&reached).cloned().collect())
        })
        .last()
        .unwrap_or_default();
        manifests
            .iter()
            .filter(|manifest| manifest.packaged.is_some())
            .map(|manifest| {
                let tested = manifest
                    .features
                    .keys()
                    .filter(|feature| {
                        !enabled.contains(&Enabled { package: manifest.directory.clone(), feature: (*feature).clone() })
                    })
                    .cloned()
                    .collect();
                (manifest.directory.clone(), tested)
            })
            .collect()
    }

    fn implied(by_directory: &BTreeMap<&Path, &Manifest>, enabled: &Enabled) -> Vec<Enabled> {
        let manifest = by_directory.get(enabled.package.as_path());
        let entries = manifest.and_then(|found| found.features.get(&enabled.feature).map(|entries| (found, entries)));
        entries.map_or_default(|(found, listed)| {
            listed
                .iter()
                .filter_map(|entry| match entry {
                    Enabling::Own(feature) => {
                        Some(Enabled { package: found.directory.clone(), feature: feature.clone() })
                    }
                    Enabling::Dependency { dependency, feature } => found
                        .named
                        .get(dependency)
                        .map(|package| Enabled { package: package.clone(), feature: feature.clone() }),
                    Enabling::Optional => None,
                })
                .collect()
        })
    }

    fn pathed<'table>(
        directory: &Path,
        sections: impl Iterator<Item = &'table toml::Spanned<DeValue<'table>>>,
    ) -> Vec<PathBuf> {
        sections
            .filter_map(|section| section.get_ref().as_table())
            .flat_map(|dependencies| dependencies.iter().map(|(_, dependency)| dependency.get_ref()))
            .filter_map(|dependency| dependency.as_table().and_then(|inner| inner.get("path")))
            .filter_map(|path| path.get_ref().as_str())
            .map(|path| Manifest::lexical(&directory.join(path)))
            .collect()
    }

    pub(crate) fn testing(manifests: &[Manifest], binaries: &BTreeSet<PathBuf>) -> BTreeSet<PathBuf> {
        let packaged: Vec<&Manifest> = manifests.iter().filter(|manifest| manifest.packaged.is_some()).collect();
        let depended: BTreeSet<&PathBuf> = packaged.iter().flat_map(|manifest| &manifest.depends).collect();
        let roots: BTreeSet<PathBuf> = packaged
            .iter()
            .map(|manifest| &manifest.directory)
            .filter(|directory| !depended.contains(directory))
            .chain(binaries)
            .cloned()
            .collect();
        let shipped = iter::successors(Some(roots), |known| {
            let reached: BTreeSet<PathBuf> = packaged
                .iter()
                .filter(|manifest| known.contains(&manifest.directory))
                .flat_map(|manifest| manifest.shipping.iter())
                .filter(|path| !known.contains(*path))
                .cloned()
                .collect();
            reached.first().map(|_| known.union(&reached).cloned().collect())
        })
        .last()
        .unwrap_or_default();
        packaged
            .iter()
            .map(|manifest| manifest.directory.clone())
            .filter(|directory| !shipped.contains(directory))
            .collect()
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

    fn lexical(path: &Path) -> PathBuf {
        path.components().fold(PathBuf::new(), |folded, component| match component {
            Component::ParentDir => folded.parent().map_or_else(|| folded.join(component), Path::to_path_buf),
            Component::CurDir => folded,
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => folded.join(component),
        })
    }
}
