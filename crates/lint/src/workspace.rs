mod manifest;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use invariants::Ignoring;
use toml::de::{DeTable, DeValue};

use crate::failure::Failure;
use crate::ignored::Ignored;
use crate::tree::Tree;
use manifest::Manifest;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CrateKind {
    Library,
    Binary,
    Build,
    Tests,
    Examples,
    Benches,
}

#[derive(Debug)]
pub(crate) struct Feature {
    name: String,
    line: usize,
}

impl Feature {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn line(&self) -> usize {
        self.line
    }
}

#[derive(Debug)]
struct Section {
    key: &'static str,
    shown: &'static str,
    kind: CrateKind,
}

#[derive(Clone, Copy, Debug)]
struct Read<'manifest> {
    directory: &'manifest Path,
    named: &'manifest str,
    text: &'manifest str,
}

#[derive(Clone, Debug)]
struct Target {
    path: PathBuf,
    kind: CrateKind,
}

#[derive(Debug)]
pub(crate) struct Package {
    directory: PathBuf,
    name: String,
    manifest: Arc<Path>,
    features: Vec<Feature>,
    unopted: Vec<Feature>,
    declared: Vec<Target>,
}

impl Package {
    pub(crate) fn directory(&self) -> &Path {
        &self.directory
    }

    pub(crate) fn crate_name(&self) -> String {
        self.name.chars().map(|letter| if letter == '-' { '_' } else { letter }).collect()
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn manifest(&self) -> &Arc<Path> {
        &self.manifest
    }

    pub(crate) fn features(&self) -> &[Feature] {
        &self.features
    }

    pub(crate) fn unopted(&self) -> &[Feature] {
        &self.unopted
    }
}

#[derive(Debug)]
pub(crate) struct CrateRoot {
    target: Target,
    package: Arc<Package>,
}

impl CrateRoot {
    pub(crate) fn path(&self) -> &Path {
        &self.target.path
    }

    pub(crate) fn kind(&self) -> CrateKind {
        self.target.kind
    }

    pub(crate) fn package(&self) -> &Arc<Package> {
        &self.package
    }
}

#[derive(Debug)]
struct Listed {
    package: Option<Package>,
    manifest: Manifest,
}

#[derive(Debug)]
pub(crate) struct Workspace {
    packages: Vec<Arc<Package>>,
    roots: Vec<CrateRoot>,
}

impl Workspace {
    const SOURCE_EXTENSION: &'static str = "rs";
    const MAIN: &'static str = "main.rs";
    const FIXED: [(&'static str, CrateKind); 3] =
        [("build.rs", CrateKind::Build), ("src/lib.rs", CrateKind::Library), ("src/main.rs", CrateKind::Binary)];
    const TARGETS: [(&'static str, CrateKind); 4] = [
        ("src/bin", CrateKind::Binary),
        ("tests", CrateKind::Tests),
        ("examples", CrateKind::Examples),
        ("benches", CrateKind::Benches),
    ];
    const OPTED: [Section; 2] = [
        Section { key: "lib", shown: "[lib]", kind: CrateKind::Library },
        Section { key: "bin", shown: "[[bin]]", kind: CrateKind::Binary },
    ];
    const LIB_PATH: &'static str = "src/lib.rs";
    const MAIN_PATH: &'static str = "src/main.rs";
    const BINS: &'static str = "src/bin";
    const DECLARED: [(&'static str, CrateKind); 5] = [
        ("lib", CrateKind::Library),
        ("bin", CrateKind::Binary),
        ("test", CrateKind::Tests),
        ("example", CrateKind::Examples),
        ("bench", CrateKind::Benches),
    ];

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

    pub(crate) fn packages(&self) -> &[Arc<Package>] {
        &self.packages
    }

    pub(crate) fn into_roots(self) -> Vec<CrateRoot> {
        self.roots
    }

    fn directory(manifest: &Path) -> PathBuf {
        manifest.parent().map_or_default(Path::to_path_buf)
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

    fn roots(tree: &Tree, package: &Arc<Package>) -> Vec<CrateRoot> {
        let directory = &package.directory;
        let fixed = Workspace::FIXED.iter().map(|(file, kind)| Target { path: directory.join(file), kind: *kind });
        let targets = Workspace::TARGETS.iter().flat_map(|(folder, kind)| {
            let inside = directory.join(folder);
            let roots = tree.below(&inside).filter(|path| Workspace::target_root(&inside, path).is_some());
            roots.map(|path| Target { path: path.to_path_buf(), kind: *kind }).collect::<Vec<Target>>()
        });
        let all = package.declared.iter().cloned().chain(fixed).chain(targets);
        all.filter(|target| tree.text(&target.path).is_some())
            .map(|target| CrateRoot { target, package: Arc::clone(package) })
            .collect()
    }

    fn target_root(folder: &Path, path: &Path) -> Option<()> {
        let parts: Vec<&OsStr> = path.strip_prefix(folder).ignoring(Ignored::Outside)?.iter().collect();
        match parts.as_slice() {
            [file] => (Path::new(file).extension() == Some(OsStr::new(Workspace::SOURCE_EXTENSION))).then_some(()),
            [_, main] => (*main == Workspace::MAIN).then_some(()),
            _ => None,
        }
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

    fn built(&self, tree: &Tree, directory: &Path) -> Vec<PathBuf> {
        let file = |relative: &str| tree.text(&directory.join(relative)).map(|_| directory.join(relative));
        match self.kind {
            CrateKind::Binary => {
                let bins = directory.join(Workspace::BINS);
                let nested: Vec<PathBuf> = tree
                    .below(&bins)
                    .filter(|path| Workspace::target_root(&bins, path).is_some())
                    .map(Path::to_path_buf)
                    .collect();
                file(Workspace::MAIN_PATH).into_iter().chain(nested).collect()
            }
            CrateKind::Library | CrateKind::Build | CrateKind::Tests | CrateKind::Examples | CrateKind::Benches => {
                file(Workspace::LIB_PATH).into_iter().collect()
            }
        }
    }
}
