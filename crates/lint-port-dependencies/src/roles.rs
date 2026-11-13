mod boundary;
mod breached;
mod expanded;
mod undenied;
mod unexpanded;

use std::borrow::Borrow;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use graph::Model;
use lint_core::{FileHead, Parsed, Site, Violation};
use lint_file::PrimedFacts;
use lint_layout::LineNumber;
use lint_layout::{Name, UsePath};
use lint_placement::{CrateName as PlacedName, Role as Placed};
use workspace_metadata::{CrateName, Expansion, Role};

use crate::owners::Owners;
use boundary::Underscored;
use breached::{Breached, Fixed};
use expanded::ModelCalls;
use undenied::Undenied;
use unexpanded::Unexpanded;

#[derive(Debug)]
pub struct RootDenial<'head> {
    package: &'head Name,
    path: &'head Path,
    denied: BTreeSet<UsePath>,
}

#[derive(Debug)]
pub struct RoleFile<'head> {
    root: Option<RootDenial<'head>>,
}

#[derive(Debug, Default)]
pub struct RoleFacts<'head> {
    denials: BTreeMap<&'head Name, RootDenial<'head>>,
}

impl<'head> FromIterator<RoleFile<'head>> for RoleFacts<'head> {
    fn from_iter<Parts: IntoIterator<Item = RoleFile<'head>>>(parts: Parts) -> RoleFacts<'head> {
        parts.into_iter().fold(RoleFacts::default(), |mut facts, part| {
            facts.denials.extend(part.root.map(|root| (root.package, root)));
            facts
        })
    }
}

#[derive(Debug)]
pub struct RoleReading;

impl PrimedFacts for RoleReading {
    type PrimerPart<'head> = ();
    type Primer<'head> = ();
    type Part<'head> = RoleFile<'head>;
    type Context<'head> = RoleFacts<'head>;

    fn primed(_parsed: Parsed<'_, '_>) {}

    fn contributed<'head>(parsed: Parsed<'_, 'head>, (): &()) -> RoleFile<'head> {
        let head = parsed.head();
        let root = Some(head).filter(|found| found.placement().role() == Placed::LibraryRoot);
        RoleFile {
            root: root.and_then(|found| {
                let package = found.placement().package().root().package()?;
                Some(RootDenial { package, path: found.path(), denied: Undenied::denied(parsed.file().syntax()) })
            }),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Roles<'file> {
    owners: Owners<'file>,
    facts: &'file RoleFacts<'file>,
}

impl<'file> Roles<'file> {
    pub(crate) fn new(heads: &'file [FileHead], facts: &'file RoleFacts<'file>) -> Roles<'file> {
        Roles { owners: Owners::declared(heads), facts }
    }

    pub(crate) fn breached<'meta, Breach: Fixed<'meta>>(&self, breaches: &[Breach]) -> Vec<Violation<'file>> {
        breaches
            .iter()
            .filter_map(|breach| Some((self.owners.manifest(breach.as_ref().dependent())?, breach)))
            .flat_map(|(file, breach)| {
                let package = file.placement().package();
                let lines = Some(breach.lines(package))
                    .filter(|found| found.first().is_some())
                    .unwrap_or_else(|| BTreeSet::from([package.dependency_sites().header()]));
                let manifest = file.manifest_file().as_ref();
                lines.into_iter().map(move |line| Violation::new(Site::new(manifest, line), Breached::new(breach)))
            })
            .collect()
    }

    pub(crate) fn undenied(&self, role: Role, served: &[&CrateName]) -> Vec<Violation<'file>> {
        let denying = |name: &&CrateName| {
            let (site, missing) = self.facts.denials.get(&Name::from(name.as_ref())).map_or_else(
                || {
                    Some((
                        Site::new(self.owners.manifest(name)?.manifest_file().as_ref(), LineNumber::FIRST),
                        Undenied::all(role),
                    ))
                },
                |root| Some((Site::new(root.path, LineNumber::FIRST), Undenied::missing(role, &root.denied))),
            )?;
            Some(missing.into_iter().map(move |undenied| Violation::new(site, undenied)))
        };
        served.iter().filter_map(denying).flatten().collect()
    }

    pub(crate) fn unexpanded(&self, expansions: &[Expansion<'file>], model: &'file Model) -> Vec<Violation<'file>> {
        let calls = ModelCalls::from(model);
        expansions.iter().map(|expansion| self.expected(expansion)).flat_map(|wanted| wanted.found_in(&calls)).collect()
    }

    fn expected(&self, expansion: &Expansion<'file>) -> Expected<'file> {
        let wanted: Vec<Vec<Name>> = expansion.paths(|member, name| {
            let placed = PlacedName::from(self.owners.manifest(member)?.placement().package().root());
            let root: &Name = placed.borrow();
            Some(vec![root.clone(), Name::from(name.as_ref())])
        });
        let admitted: BTreeSet<Name> = expansion.admitted().map(Underscored::crate_named).collect();
        Expected { wanted, admitted, finding: Unexpanded::new(expansion.port(), expansion.expanders().role()) }
    }
}

#[derive(Debug)]
struct Expected<'file> {
    wanted: Vec<Vec<Name>>,
    admitted: BTreeSet<Name>,
    finding: Unexpanded<'file>,
}

impl<'file> Expected<'file> {
    fn found_in(&self, calls: &ModelCalls<'file>) -> Vec<Violation<'file>> {
        calls.sites(&self.wanted, &self.admitted).into_iter().map(|site| Violation::new(site, self.finding)).collect()
    }
}
