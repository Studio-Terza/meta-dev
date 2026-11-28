mod boundary;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use invariants::Ignoring;

use crate::ignored::Ignored;
use crate::tree::Tree;

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

    pub(crate) fn packages(&self) -> &[Arc<Package>] {
        &self.packages
    }

    pub(crate) fn into_roots(self) -> Vec<CrateRoot> {
        self.roots
    }

    fn directory(manifest: &Path) -> PathBuf {
        manifest.parent().map_or_default(Path::to_path_buf)
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
