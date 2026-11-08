use std::collections::{BTreeMap, BTreeSet};

use lint_layout::{LibraryForm, Name, PackageName};

use crate::domain::Domain;
use crate::external::External;
use crate::item_path::ItemPath;
use crate::resolved::Resolved;
use crate::unresolved::Unresolved;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reexport {
    External(External),
    Own,
    Opaque,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Found {
    Through(External),
    Own,
    Absent,
    Opaque,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Globs(Vec<Reexport>);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CrateExports {
    named: BTreeMap<Name, Vec<Reexport>>,
    declared: BTreeSet<Name>,
    globs: Globs,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Root {
    Read(CrateExports),
    ProcMacro(CrateExports),
    Unread,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CrateRoots(BTreeMap<PackageName, Root>);

impl Reexport {
    #[must_use]
    pub fn dependency(package: Option<&PackageName>, path: &ItemPath) -> Reexport {
        package.map_or(Reexport::Opaque, |found| Reexport::External(External::new(Some(found.clone()), path.clone())))
    }

    #[must_use]
    pub fn own(rest: &[Name], declared: &BTreeSet<&Name>) -> Reexport {
        rest.split_first()
            .filter(|(_, beyond)| beyond.is_empty())
            .and_then(|(only, _)| declared.get(only))
            .map_or(Reexport::Opaque, |_| Reexport::Own)
    }

    fn bound(bound: &[&Reexport]) -> Option<Found> {
        let (first, others) = bound.split_first()?;
        Some(others.first().map_or_else(|| first.found(), |_| Found::Opaque))
    }

    fn found(&self) -> Found {
        match self {
            Reexport::External(target) => Found::Through(target.clone()),
            Reexport::Own => Found::Own,
            Reexport::Opaque => Found::Opaque,
        }
    }

    fn globbed(&self, name: &Name, inside: &impl Fn(&PackageName, &Name) -> Found) -> Found {
        match self {
            Reexport::External(source) => Found::globbed_from(source, name, inside),
            Reexport::Own | Reexport::Opaque => Found::Opaque,
        }
    }

    fn package(&self) -> Option<&PackageName> {
        match self {
            Reexport::External(target) => target.package(),
            Reexport::Own | Reexport::Opaque => None,
        }
    }
}

impl Found {
    fn globbed_from(source: &External, name: &Name, inside: &impl Fn(&PackageName, &Name) -> Found) -> Found {
        source
            .package()
            .filter(|_| source.below_root().is_empty())
            .map_or(Found::Opaque, |package| inside(package, name).seen_through(source, name))
    }

    fn seen_through(self, source: &External, name: &Name) -> Found {
        match self {
            Found::Own => Found::Through(source.joined(name)),
            Found::Through(_) | Found::Absent | Found::Opaque => self,
        }
    }

    fn resolved(self, rest: &[Name], written: &External) -> Option<Resolved> {
        match self {
            Found::Through(target) => {
                Some(Resolved::External(rest.iter().fold(target, |path, segment| path.joined(segment))))
            }
            Found::Own => None,
            Found::Absent | Found::Opaque => Some(Resolved::Unknown(written.path().clone(), Unresolved::Unread)),
        }
    }
}

impl Globs {
    fn found(&self, name: &Name, inside: &impl Fn(&PackageName, &Name) -> Found) -> Found {
        let found: Vec<Found> =
            self.0.iter().map(|glob| glob.globbed(name, inside)).filter(|seen| *seen != Found::Absent).collect();
        let definite: Vec<&Found> = found.iter().filter(|seen| **seen != Found::Opaque).collect();
        definite
            .split_first()
            .filter(|(_, others)| others.is_empty())
            .map(|(only, _)| (*only).clone())
            .or_else(|| found.first().map(|_| Found::Opaque))
            .unwrap_or(Found::Absent)
    }
}

impl CrateExports {
    #[must_use]
    pub fn with_named(mut self, name: Name, reexport: Reexport) -> CrateExports {
        self.named.entry(name).or_default().push(reexport);
        self
    }

    #[must_use]
    pub fn with_declared(mut self, name: Name) -> CrateExports {
        self.declared.insert(name);
        self
    }

    #[must_use]
    pub fn with_glob(mut self, reexport: Reexport) -> CrateExports {
        self.globs.0.push(reexport);
        self
    }

    pub fn packages(&self) -> impl Iterator<Item = &PackageName> {
        self.named.values().flatten().chain(&self.globs.0).filter_map(Reexport::package)
    }

    fn found(
        &self,
        name: &Name,
        chosen: &impl Fn(&[Reexport]) -> Vec<&Reexport>,
        inside: &impl Fn(&PackageName, &Name) -> Found,
    ) -> Found {
        self.named
            .get(name)
            .map(|bound| chosen(bound))
            .and_then(|bound| Reexport::bound(&bound))
            .unwrap_or_else(|| self.declared.get(name).map_or_else(|| self.globs.found(name, inside), |_| Found::Own))
    }
}

impl Root {
    #[must_use]
    pub fn formed(form: LibraryForm, exports: CrateExports) -> Root {
        match form {
            LibraryForm::Rust => Root::Read(exports),
            LibraryForm::ProcMacro => Root::ProcMacro(exports),
        }
    }

    pub fn packages(&self) -> impl Iterator<Item = &PackageName> {
        self.exports().into_iter().flat_map(CrateExports::packages)
    }

    fn exports(&self) -> Option<&CrateExports> {
        match self {
            Root::Read(exports) | Root::ProcMacro(exports) => Some(exports),
            Root::Unread => None,
        }
    }

    fn macros(&self) -> Option<&Root> {
        match self {
            Root::ProcMacro(_) => Some(self),
            Root::Read(_) | Root::Unread => None,
        }
    }
}

impl FromIterator<(PackageName, Root)> for CrateRoots {
    fn from_iter<Read: IntoIterator<Item = (PackageName, Root)>>(read: Read) -> CrateRoots {
        CrateRoots(read.into_iter().collect())
    }
}

impl CrateRoots {
    #[must_use]
    pub fn root(&self, package: &PackageName) -> Option<&Root> {
        self.0.get(package)
    }

    #[must_use]
    pub fn with_root(mut self, package: PackageName, root: Root) -> CrateRoots {
        self.0.insert(package, root);
        self
    }

    #[must_use]
    pub fn followed(&self, external: &External) -> Option<Resolved> {
        let package = external.package()?;
        let (name, rest) = external.below_root().split_first()?;
        let domain = rest.first().map_or(Domain::Macros, |_| Domain::Types);
        self.found(package, name, domain, &BTreeSet::new()).resolved(rest, external)
    }

    fn found(&self, package: &PackageName, name: &Name, domain: Domain, seen: &BTreeSet<PackageName>) -> Found {
        let deeper: BTreeSet<PackageName> = seen.iter().chain([package]).cloned().collect();
        self.0.get(package).filter(|_| seen.get(package).is_none()).and_then(Root::exports).map_or(
            Found::Opaque,
            |exports| {
                exports.found(name, &|bound| self.chosen(bound, domain), &|glob, inner| {
                    self.found(glob, inner, domain, &deeper)
                })
            },
        )
    }

    fn chosen<'bound>(&self, bound: &'bound [Reexport], domain: Domain) -> Vec<&'bound Reexport> {
        let (macros, others): (Vec<&Reexport>, Vec<&Reexport>) = bound
            .iter()
            .partition(|reexport| reexport.package().and_then(|package| self.0.get(package)?.macros()).is_some());
        domain.chosen(macros, others)
    }
}
