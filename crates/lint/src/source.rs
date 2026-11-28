use std::path::Path;
use std::sync::Arc;

use invariants::Ignoring;

use crate::ignored::Ignored;
use crate::terms::{Name, Text};
use crate::workspace::{CrateKind, Package};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Place {
    Shipped,
    Tests,
    Elsewhere,
}

#[derive(Debug)]
struct Location {
    kind: CrateKind,
    library: Option<()>,
}

#[derive(Clone, Debug)]
pub(crate) struct ModulePath {
    root: Arc<Path>,
    names: Vec<Name>,
}

#[derive(Debug)]
pub(crate) struct Source {
    real: Arc<Path>,
    shown: Arc<Path>,
    text: Text,
    package: Arc<Package>,
    location: Location,
    module: ModulePath,
    reached: CrateKind,
}

impl Location {
    const SOURCE: &'static str = "src";
    const LIBRARY: &'static str = "lib.rs";
    const TARGETS: [(&'static str, CrateKind); 3] =
        [("tests", CrateKind::Tests), ("examples", CrateKind::Examples), ("benches", CrateKind::Benches)];

    fn of(package: &Package, real: &Path, fallback: CrateKind) -> Location {
        let inside = real.strip_prefix(package.directory()).ignoring(Ignored::Outside).unwrap_or(real);
        let parts: Vec<Name> =
            inside.components().map(|component| Name::spelled(&component.as_os_str().to_string_lossy())).collect();
        let target = parts.first().and_then(|first| Location::TARGETS.iter().find(|(name, _)| first == *name));
        match (parts.as_slice(), target) {
            ([source, rest @ ..], _) if source == Location::SOURCE => Location {
                kind: CrateKind::Library,
                library: matches!(rest, [only] if only == Location::LIBRARY).then_some(()),
            },
            (_, Some((_, kind))) => Location { kind: *kind, library: None },
            _ => Location { kind: fallback, library: None },
        }
    }
}

impl ModulePath {
    pub(crate) fn root(root: Arc<Path>) -> ModulePath {
        ModulePath { root, names: Vec::new() }
    }

    pub(crate) fn child(&self, names: impl IntoIterator<Item = Name>) -> ModulePath {
        ModulePath { root: Arc::clone(&self.root), names: self.names.iter().cloned().chain(names).collect() }
    }

    pub(crate) fn crate_root(&self) -> &Arc<Path> {
        &self.root
    }

    pub(crate) fn names(&self) -> &[Name] {
        &self.names
    }
}

impl Source {
    const PORTS: &'static str = "ports";
    const BOUNDARY: &'static str = "boundary";

    pub(crate) fn bounded(&self) -> Option<()> {
        self.module.names.iter().any(|segment| segment == Source::BOUNDARY).then_some(())
    }

    pub(crate) fn new(
        package: Arc<Package>,
        real: Arc<Path>,
        shown: Arc<Path>,
        module: ModulePath,
        reached: CrateKind,
        text: Text,
    ) -> Source {
        let location = Location::of(&package, &real, reached);
        Source { real, shown, text, package, location, module, reached }
    }

    pub(crate) fn module(&self) -> &ModulePath {
        &self.module
    }

    pub(crate) fn real(&self) -> &Path {
        &self.real
    }

    pub(crate) fn shown(&self) -> &Arc<Path> {
        &self.shown
    }

    pub(crate) fn text(&self) -> &Text {
        &self.text
    }

    pub(crate) fn package(&self) -> &Arc<Package> {
        &self.package
    }

    pub(crate) fn place(&self) -> Place {
        match self.location.kind {
            CrateKind::Library | CrateKind::Binary | CrateKind::Build => Place::Shipped,
            CrateKind::Tests => Place::Tests,
            CrateKind::Examples | CrateKind::Benches => Place::Elsewhere,
        }
    }

    pub(crate) fn library_root(&self) -> Option<()> {
        self.location.library
    }

    pub(crate) fn own_crate(&self) -> Option<Name> {
        (self.place() == Place::Shipped && self.reached != CrateKind::Binary).then(|| self.package.crate_name())
    }

    pub(crate) fn ported(&self) -> Option<()> {
        let last_word = self
            .package
            .name()
            .text()
            .chars()
            .rev()
            .skip_while(|letter| !letter.is_alphanumeric())
            .take_while(|letter| letter.is_alphanumeric())
            .flat_map(char::to_lowercase);
        let named = self.place() == Place::Shipped && last_word.eq(Source::PORTS.chars().rev());
        let module = self.module.names.iter().any(|segment| segment == Source::PORTS);
        (named || module).then_some(())
    }
}
