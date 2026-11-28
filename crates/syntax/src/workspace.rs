mod manifest;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use invariants::Ignoring;
use toml_ports::{Document, Table, Value};

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
            let document = Document::read(tree.text(manifest)?).ignoring(Ignored::Unparsed)?;
            Some(Manifest::new(Workspace::directory(manifest), document.table()))
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
        let document = Document::read(text)
            .map_err(|source| Failure::Manifest { path: tree.shown(manifest).to_path_buf(), source })?;
        let table = document.table();
        let directory = Workspace::directory(manifest);
        let name = table.get("package").and_then(|package| Workspace::string(package, "name"));
        let package = name.map(|named| Package {
            name: named.to_owned(),
            manifest: tree.shown(manifest),
            features: Workspace::features(table),
            declared: Workspace::declared(&directory, table),
            directory: directory.clone(),
        });
        Ok(Listed { package, manifest: Manifest::new(directory, table) })
    }

    fn string<'value>(value: Value<'value, '_>, key: &str) -> Option<&'value str> {
        value.table().and_then(|table| table.get(key)).and_then(Value::text)
    }

    fn features(table: Table<'_, '_>) -> Vec<Feature> {
        let declared = table.get("features").and_then(Value::table);
        declared.map_or_default(|features| {
            features.keys().into_iter().map(|key| Feature { name: key.name().to_owned(), line: key.line() }).collect()
        })
    }

    fn declared(directory: &Path, table: Table<'_, '_>) -> Vec<Target> {
        Workspace::DECLARED
            .iter()
            .flat_map(|(section, kind)| {
                table
                    .get(section)
                    .map_or_default(Value::each)
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
