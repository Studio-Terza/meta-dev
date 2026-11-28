mod manifest;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use invariants::Ignoring;
use toml::de::{DeTable, DeValue};

use super::{CrateKind, CrateRoot, Feature, Package, Read, Section, Target, Workspace};
use crate::failure::Failure;
use crate::ignored::Ignored;
use crate::tree::Tree;
use manifest::Manifest;

#[derive(Debug)]
struct Listed {
    package: Option<Package>,
    manifest: Manifest,
}

impl Workspace {
    pub(crate) fn found(tree: &Tree) -> Result<Workspace, Failure> {
        let (inside, above): (Vec<&Path>, Vec<&Path>) =
            tree.manifests().partition(|manifest| manifest.starts_with(tree.root()));
        let read: Vec<Listed> =
            inside.into_iter().map(|manifest| Workspace::listed(tree, manifest)).collect::<Result<_, _>>()?;
        let outer = above.into_iter().filter_map(|manifest| {
            let table = DeTable::parse(tree.text(manifest)?).ignoring(Ignored::Unparsed)?;
            Some(Manifest::new(Workspace::directory(manifest), table.get_ref()))
        });
        let (found, listed): (Vec<Option<Package>>, Vec<Manifest>) =
            read.into_iter().map(|entry| (entry.package, entry.manifest)).unzip();
        let admitted = Manifest::admitted(&listed.into_iter().chain(outer).collect::<Vec<Manifest>>());
        let packages: Vec<Arc<Package>> =
            found.into_iter().flatten().filter(|package| admitted.contains(&package.directory)).map(Arc::new).collect();
        let roots: Vec<CrateRoot> = packages.iter().flat_map(|package| Workspace::roots(tree, package)).collect();
        match roots.first() {
            Some(_) => Ok(Workspace { packages, roots }),
            None => Err(tree.empty()),
        }
    }

    fn listed(tree: &Tree, manifest: &Path) -> Result<Listed, Failure> {
        let text = tree.text(manifest).unwrap_or_default();
        let table = DeTable::parse(text)
            .map_err(|source| Failure::Manifest { path: tree.shown(manifest).to_path_buf(), source })?
            .into_inner();
        let directory = Workspace::directory(manifest);
        let name = table.get("package").and_then(|package| Workspace::string(package.get_ref(), "name"));
        let package = name.map(|named| Package {
            name: named.to_owned(),
            manifest: tree.shown(manifest),
            features: Workspace::features(text, &table),
            unopted: Workspace::unopted(tree, Read { directory: &directory, named, text }, &table),
            declared: Workspace::declared(&directory, &table),
            directory: directory.clone(),
        });
        Ok(Listed { package, manifest: Manifest::new(directory, &table) })
    }

    fn string<'value>(value: &'value DeValue<'_>, key: &str) -> Option<&'value str> {
        value.as_table().and_then(|table| table.get(key)).and_then(|found| found.get_ref().as_str())
    }

    fn features(text: &str, table: &DeTable<'_>) -> Vec<Feature> {
        let declared = table.get("features").and_then(|features| features.get_ref().as_table());
        declared.map_or_default(|features| {
            features
                .iter()
                .map(|(key, _)| Feature {
                    name: key.get_ref().to_string(),
                    line: Workspace::line(text, key.span().start),
                })
                .collect()
        })
    }

    fn line(text: &str, start: usize) -> usize {
        text.get(..start).map_or(0, |before| before.matches('\n').count()) + 1
    }

    fn unopted(tree: &Tree, read: Read<'_>, table: &DeTable<'_>) -> Vec<Feature> {
        Workspace::OPTED.iter().flat_map(|section| section.unopted(tree, read, table)).collect()
    }

    fn declared(directory: &Path, table: &DeTable<'_>) -> Vec<Target> {
        Workspace::DECLARED
            .iter()
            .flat_map(|(section, kind)| {
                let value = table.get(*section).map(toml::Spanned::get_ref);
                let tables: Vec<&DeValue<'_>> = match value {
                    Some(DeValue::Array(listed)) => listed.iter().map(toml::Spanned::get_ref).collect(),
                    Some(single) => vec![single],
                    None => Vec::new(),
                };
                tables
                    .into_iter()
                    .filter_map(|target| Workspace::string(target, "path"))
                    .map(|path| Target { path: directory.join(path), kind: *kind })
                    .collect::<Vec<Target>>()
            })
            .collect()
    }
}

impl Section {
    const OPT_OUTS: [&'static str; 3] = ["test", "doctest", "bench"];

    fn unopted(&self, tree: &Tree, read: Read<'_>, table: &DeTable<'_>) -> Vec<Feature> {
        let Read { directory, named, text } = read;
        let found = table.iter().find(|(key, _)| key.get_ref() == self.key);
        let entries: Vec<&DeTable<'_>> = found.map_or_default(|(_, value)| match value.get_ref() {
            DeValue::Array(listed) => listed.iter().filter_map(|entry| entry.get_ref().as_table()).collect(),
            single => single.as_table().into_iter().collect(),
        });
        let declared: Vec<PathBuf> = entries.iter().flat_map(|entry| self.paths(directory, named, entry)).collect();
        let missing = self.built(tree, directory).into_iter().filter(|root| !declared.contains(root)).map(|root| {
            let file = root.strip_prefix(directory).ignoring(Ignored::Outside).unwrap_or(&root).display().to_string();
            Feature { name: format!("{file} is built with no {} section", self.shown), line: 1 }
        });
        let unopted = entries.into_iter().filter(|entry| {
            !Section::OPT_OUTS
                .iter()
                .all(|key| entry.get(*key).and_then(|value| value.get_ref().as_bool()) == Some(false))
        });
        let lined = unopted.map(|entry| {
            let first = entry.iter().map(|(key, _)| key.span().start).min().unwrap_or_default();
            Feature {
                name: format!("{} target without test, doctest and bench set false", self.shown),
                line: Workspace::line(text, first),
            }
        });
        missing.chain(lined).collect()
    }

    fn paths(&self, directory: &Path, named: &str, entry: &DeTable<'_>) -> Vec<PathBuf> {
        let given = entry.get("path").and_then(|path| path.get_ref().as_str());
        let name = entry.get("name").and_then(|name| name.get_ref().as_str()).unwrap_or(named);
        given.map_or_else(
            || match self.kind {
                CrateKind::Binary => vec![
                    directory.join(Workspace::MAIN_PATH),
                    directory.join(Workspace::BINS).join(format!("{name}.{}", Workspace::SOURCE_EXTENSION)),
                    directory.join(Workspace::BINS).join(name).join(Workspace::MAIN),
                ],
                CrateKind::Library | CrateKind::Build | CrateKind::Tests | CrateKind::Examples | CrateKind::Benches => {
                    vec![directory.join(Workspace::LIB_PATH)]
                }
            },
            |path| vec![directory.join(path)],
        )
    }
}
