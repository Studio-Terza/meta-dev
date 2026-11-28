mod boundary;
mod manifest;

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::iter;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use file_ports::{Files, Kind};
use invariants::Ignoring;
use toml::de::{DeTable, DeValue};

use crate::failure::Failure;
use crate::ignored::Ignored;
use crate::terms::{Line, Name};
use manifest::Manifest;

#[derive(Debug)]
pub(crate) struct Tree {
    given: PathBuf,
    canonical: PathBuf,
}

impl Tree {
    const CURRENT: &'static str = ".";

    pub(crate) fn opened(files: &impl Files, given: &Path) -> Result<Tree, Failure> {
        let canonical = files.canonical(given).map_err(|source| Failure::Read { path: given.to_path_buf(), source })?;
        Ok(Tree { given: given.to_path_buf(), canonical })
    }

    pub(crate) fn shown(&self, real: &Path) -> Arc<Path> {
        let inside = real.strip_prefix(&self.canonical).ignoring(Ignored::Outside);
        let shown = inside.map_or_else(
            || real.to_path_buf(),
            |relative| {
                if self.given == Path::new(Tree::CURRENT) { relative.to_path_buf() } else { self.given.join(relative) }
            },
        );
        Arc::from(shown)
    }

    pub(crate) fn empty(&self) -> Failure {
        Failure::NoSources { path: self.given.clone() }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CrateKind {
    Library,
    Binary,
    Build,
    Tests,
    Examples,
    Benches,
}

#[derive(Clone, Debug)]
pub(crate) struct Feature {
    name: Name,
    line: Line,
}

impl Feature {
    pub(crate) fn name(&self) -> &Name {
        &self.name
    }

    pub(crate) fn line(&self) -> Line {
        self.line
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Needed {
    Shipped,
    Tests,
}

#[derive(Clone, Debug)]
pub(crate) struct Dependency {
    name: Name,
    needed: Needed,
    line: Line,
}

impl Dependency {
    pub(crate) fn name(&self) -> &Name {
        &self.name
    }

    pub(crate) fn needed(&self) -> Needed {
        self.needed
    }

    pub(crate) fn line(&self) -> Line {
        self.line
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Package {
    directory: PathBuf,
    name: Name,
    manifest: Arc<Path>,
    features: Vec<Feature>,
    dependencies: Vec<Dependency>,
    testing: Option<()>,
    test_features: BTreeSet<Name>,
}

impl Package {
    fn placed(self, testing: &BTreeSet<PathBuf>, test_features: &BTreeMap<PathBuf, BTreeSet<Name>>) -> Package {
        Package {
            testing: testing.contains(&self.directory).then_some(()),
            test_features: test_features.get(&self.directory).cloned().unwrap_or_default(),
            ..self
        }
    }

    pub(crate) fn test_features(&self) -> &BTreeSet<Name> {
        &self.test_features
    }

    pub(crate) fn shipped(&self) -> Option<()> {
        self.testing.xor(Some(()))
    }

    pub(crate) fn directory(&self) -> &Path {
        &self.directory
    }

    pub(crate) fn crate_name(&self) -> Name {
        self.name.as_crate()
    }

    pub(crate) fn name(&self) -> &Name {
        &self.name
    }

    pub(crate) fn manifest(&self) -> &Arc<Path> {
        &self.manifest
    }

    pub(crate) fn features(&self) -> &[Feature] {
        &self.features
    }

    pub(crate) fn dependencies(&self) -> &[Dependency] {
        &self.dependencies
    }
}

#[derive(Debug)]
pub(crate) struct CrateRoot {
    path: PathBuf,
    kind: CrateKind,
    package: Arc<Package>,
}

impl CrateRoot {
    fn new(path: PathBuf, kind: CrateKind, package: &Arc<Package>) -> CrateRoot {
        CrateRoot { path, kind, package: Arc::clone(package) }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn kind(&self) -> CrateKind {
        self.kind
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
    const MANIFEST: &'static str = "Cargo.toml";
    const BUILD_OUTPUT: &'static str = "target";
    const HIDDEN: u8 = b'.';
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

    pub(crate) fn found(files: &impl Files, tree: &Tree) -> Result<Workspace, Failure> {
        let manifests: Vec<PathBuf> = Workspace::directories(files, &tree.canonical)
            .into_iter()
            .map(|directory| directory.join(Workspace::MANIFEST))
            .filter(|manifest| files.kind(manifest) == Kind::File)
            .collect();
        let read: Vec<Listed> =
            manifests.iter().map(|manifest| Workspace::listed(files, tree, manifest)).collect::<Result<_, _>>()?;
        let above = tree.canonical.ancestors().skip(1).filter_map(|directory| {
            let text = files.read_text(&directory.join(Workspace::MANIFEST)).ignoring(Ignored::Unlisted)?;
            let table = DeTable::parse(text.as_ref()).ignoring(Ignored::Unparsed)?;
            Some(Manifest::new(directory.to_path_buf(), table.get_ref()))
        });
        let (found, listed): (Vec<Option<Package>>, Vec<Manifest>) =
            read.into_iter().map(|entry| (entry.package, entry.manifest)).unzip();
        let read_manifests: Vec<Manifest> = listed.into_iter().chain(above).collect();
        let admitted = Manifest::admitted(&read_manifests);
        let drafts: Vec<Arc<Package>> =
            found.into_iter().flatten().filter(|package| admitted.contains(&package.directory)).map(Arc::new).collect();
        let binaries: BTreeSet<PathBuf> = drafts
            .iter()
            .flat_map(|package| Workspace::roots(files, package))
            .filter(|root| root.kind == CrateKind::Binary)
            .map(|root| root.package.directory.clone())
            .collect();
        let testing = Manifest::testing(&read_manifests, &binaries);
        let test_features = Manifest::test_features(&read_manifests, &testing);
        let packages: Vec<Arc<Package>> = drafts
            .into_iter()
            .map(|draft| Arc::new(Arc::unwrap_or_clone(draft).placed(&testing, &test_features)))
            .collect();
        let roots: Vec<CrateRoot> = packages.iter().flat_map(|package| Workspace::roots(files, package)).collect();
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

    fn directories(files: &impl Files, directory: &Path) -> Vec<PathBuf> {
        let mut entries = files.entries(directory).ignoring(Ignored::Unlisted).unwrap_or_default();
        entries.sort();
        let manifested = files.kind(&directory.join(Workspace::MANIFEST));
        let below = entries
            .into_iter()
            .filter(|entry| files.kind(entry) == Kind::Directory)
            .filter(|entry| {
                entry.file_name().and_then(|name| name.as_encoded_bytes().first().copied()) != Some(Workspace::HIDDEN)
            })
            .filter(|entry| manifested != Kind::File || entry.file_name() != Some(OsStr::new(Workspace::BUILD_OUTPUT)));
        iter::once(directory.to_path_buf())
            .chain(below.flat_map(|entry| Workspace::directories(files, &entry)))
            .collect()
    }

    fn listed(files: &impl Files, tree: &Tree, manifest: &Path) -> Result<Listed, Failure> {
        let text =
            files.read_text(manifest).map_err(|source| Failure::Read { path: manifest.to_path_buf(), source })?;
        let table = DeTable::parse(text.as_ref())
            .map_err(|source| Failure::Manifest { path: manifest.to_path_buf(), source })?
            .into_inner();
        let directory = manifest.parent().map_or_default(Path::to_path_buf);
        let name = table.get("package").and_then(|package| Workspace::named(package.get_ref()));
        let package = name.map(|named| Package {
            directory: directory.clone(),
            name: named,
            manifest: tree.shown(manifest),
            features: Workspace::features(text.as_ref(), &table),
            dependencies: Workspace::dependencies(text.as_ref(), &table),
            testing: None,
            test_features: BTreeSet::new(),
        });
        Ok(Listed { package, manifest: Manifest::new(directory, &table) })
    }

    fn roots(files: &impl Files, package: &Arc<Package>) -> Vec<CrateRoot> {
        let directory = &package.directory;
        let declared = Workspace::declared(files, package);
        let fixed = Workspace::FIXED.iter().map(|(file, kind)| CrateRoot::new(directory.join(file), *kind, package));
        let targets = Workspace::TARGETS.iter().flat_map(|(folder, kind)| {
            let mut entries = files.entries(&directory.join(folder)).ignoring(Ignored::Unlisted).unwrap_or_default();
            entries.sort();
            let roots = entries.into_iter().filter_map(|entry| Workspace::target_root(files, entry));
            roots.map(|root| CrateRoot::new(root, *kind, package)).collect::<Vec<CrateRoot>>()
        });
        declared.into_iter().chain(fixed).chain(targets).filter(|root| files.kind(&root.path) == Kind::File).collect()
    }

    fn target_root(files: &impl Files, entry: PathBuf) -> Option<PathBuf> {
        match files.kind(&entry) {
            Kind::Directory => Some(entry.join(Workspace::MAIN)),
            Kind::File | Kind::Link | Kind::Missing => {
                Some(entry).filter(|file| file.extension() == Some(OsStr::new(Workspace::SOURCE_EXTENSION)))
            }
        }
    }

    fn declared(files: &impl Files, package: &Arc<Package>) -> Vec<CrateRoot> {
        let manifest = package.directory.join(Workspace::MANIFEST);
        let text = files.read_text(&manifest).ignoring(Ignored::Unlisted).unwrap_or_default();
        let table = DeTable::parse(text.as_ref()).ignoring(Ignored::Unparsed).map(toml::Spanned::into_inner);
        table.map_or_default(|parsed| {
            Workspace::DECLARED
                .iter()
                .flat_map(|(section, kind)| {
                    let value = parsed.get(*section).map(toml::Spanned::get_ref);
                    let tables: Vec<&DeValue<'_>> = match value {
                        Some(DeValue::Array(listed)) => listed.iter().map(toml::Spanned::get_ref).collect(),
                        Some(single) => vec![single],
                        None => Vec::new(),
                    };
                    tables
                        .into_iter()
                        .filter_map(Workspace::pathed)
                        .map(|path| CrateRoot::new(package.directory.join(path), *kind, package))
                        .collect::<Vec<CrateRoot>>()
                })
                .collect()
        })
    }
}
