use std::path::Path;
use std::sync::Arc;

use invariants::Ignoring;

use crate::ignored::Ignored;
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
    module: Vec<String>,
    library: Option<()>,
}

#[derive(Debug)]
pub(crate) struct Source {
    real: Arc<Path>,
    shown: Arc<Path>,
    text: String,
    package: Arc<Package>,
    location: Location,
    reached: CrateKind,
}

impl Location {
    const SOURCE: &'static str = "src";
    const LIBRARY: &'static str = "lib.rs";
    const MODULE: &'static str = "mod";
    const EXTENSION: &'static str = "rs";
    const TARGETS: [(&'static str, CrateKind); 3] =
        [("tests", CrateKind::Tests), ("examples", CrateKind::Examples), ("benches", CrateKind::Benches)];

    fn of(package: &Package, real: &Path, fallback: CrateKind) -> Location {
        let inside = real.strip_prefix(package.directory()).ignoring(Ignored::Outside).unwrap_or(real);
        let parts: Vec<String> =
            inside.components().map(|component| component.as_os_str().to_string_lossy().into_owned()).collect();
        let target = parts.first().and_then(|first| Location::TARGETS.iter().find(|(name, _)| first == name));
        match (parts.as_slice(), target) {
            ([source, rest @ ..], _) if source == Location::SOURCE => Location {
                kind: CrateKind::Library,
                module: Location::modules(rest),
                library: (rest == [Location::LIBRARY]).then_some(()),
            },
            ([_, rest @ ..], Some((_, kind))) => {
                Location { kind: *kind, module: Location::modules(rest.get(1..).unwrap_or_default()), library: None }
            }
            _ => Location { kind: fallback, module: Vec::new(), library: None },
        }
    }

    fn modules(parts: &[String]) -> Vec<String> {
        let named = parts.iter().map(|part| {
            let path = Path::new(part);
            let source = path.extension().filter(|extension| *extension == Location::EXTENSION);
            source
                .and_then(|_| path.file_stem())
                .map_or_else(|| part.clone(), |stem| stem.to_string_lossy().into_owned())
        });
        named.filter(|name| name != Location::MODULE).collect()
    }
}

impl Source {
    const PORTS: &'static str = "ports";
    const BOUNDARY: &'static str = "boundary";

    pub(crate) fn bounded(&self) -> Option<()> {
        self.location.module.iter().any(|segment| segment == Source::BOUNDARY).then_some(())
    }

    pub(crate) fn new(
        package: Arc<Package>,
        real: Arc<Path>,
        shown: Arc<Path>,
        reached: CrateKind,
        text: String,
    ) -> Source {
        let location = Location::of(&package, &real, reached);
        Source { real, shown, text, package, location, reached }
    }

    pub(crate) fn real(&self) -> &Path {
        &self.real
    }

    pub(crate) fn shown(&self) -> &Arc<Path> {
        &self.shown
    }

    pub(crate) fn text(&self) -> &str {
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

    pub(crate) fn own_crate(&self) -> Option<String> {
        (self.place() == Place::Shipped && self.reached != CrateKind::Binary).then(|| self.package.crate_name())
    }

    pub(crate) fn ported(&self) -> Option<()> {
        let last_word = self
            .package
            .name()
            .chars()
            .rev()
            .skip_while(|letter| !letter.is_alphanumeric())
            .take_while(|letter| letter.is_alphanumeric())
            .flat_map(char::to_lowercase);
        let named = self.place() == Place::Shipped && last_word.eq(Source::PORTS.chars().rev());
        let module = self.location.module.iter().any(|segment| segment == Source::PORTS);
        (named || module).then_some(())
    }
}
