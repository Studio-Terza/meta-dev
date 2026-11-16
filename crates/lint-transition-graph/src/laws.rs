use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use graph::{ItemKind, ItemPath, LawKind, Model, OperationOwner, Placed, Resolved, Route as Written, Side, TypeRef};
use lint_core::{Finding, Fix, Site};
use lint_layout::Name;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum LawOwner {
    Named(Name),
    Operand,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LawPath {
    owner: LawOwner,
    member: Name,
}

#[derive(Clone, Debug)]
pub(crate) struct Route {
    side: Side,
    path: Option<LawPath>,
}

#[derive(Debug)]
pub(crate) struct Law<'file> {
    placed: Placed<'file, &'file graph::Law>,
    site: Site<'file>,
    routes: Vec<Route>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Unresolved<'file> {
    site: Site<'file>,
    name: Name,
}

#[derive(Debug, Default)]
pub(crate) struct Laws<'file> {
    laws: Vec<Law<'file>>,
    krates: BTreeMap<&'file Path, &'file Name>,
}

#[derive(Clone, Copy, Debug)]
struct Owners<'model> {
    model: &'model Model,
}

impl<'file> From<&'file Model> for Laws<'file> {
    fn from(model: &'file Model) -> Laws<'file> {
        let owners = Owners { model };
        Laws {
            laws: model
                .laws()
                .map(|placed| Law {
                    placed,
                    site: placed.site(),
                    routes: placed.entity().routes().iter().map(|route| owners.route(route)).collect(),
                })
                .collect(),
            krates: model.functions().map(|placed| (placed.site().path(), placed.krate())).collect(),
        }
    }
}

impl<'file> Laws<'file> {
    pub(crate) fn routing(&self, path: &Path, owner: &Name, member: &Name) -> Vec<&Law<'file>> {
        self.of_package(path).into_iter().filter(|law| law.routed(owner, member).is_some()).collect()
    }

    pub(crate) fn of_package(&self, path: &Path) -> Vec<&Law<'file>> {
        self.krates
            .get(path)
            .map_or_default(|krate| self.laws.iter().filter(|law| law.placed.krate() == *krate).collect())
    }

    pub(crate) fn of_crates(&self, paths: &[&Path]) -> impl Iterator<Item = &Law<'file>> {
        let krates: Vec<&Name> = paths.iter().filter_map(|path| self.krates.get(path).copied()).collect();
        self.laws.iter().filter(move |law| krates.contains(&law.placed.krate()))
    }
}

impl<'file> Law<'file> {
    pub(crate) fn kind(&self) -> LawKind {
        self.placed.entity().kind()
    }

    pub(crate) fn routed(&self, owner: &Name, member: &Name) -> Option<&LawPath> {
        self.routes.iter().find_map(|route| route.reaching(owner, member))
    }

    pub(crate) fn routed_on(&self, side: Side, owner: &Name, member: &Name) -> Option<&LawPath> {
        self.routes.iter().filter(|route| route.side == side).find_map(|route| route.reaching(owner, member))
    }

    pub(crate) fn on_side(&self, side: Side) -> Option<&Route> {
        self.routes.iter().find(|route| route.side == side)
    }

    pub(crate) fn unresolved(&self) -> Option<Unresolved<'file>> {
        self.routes
            .iter()
            .find(|route| route.path.is_none())
            .map(|_| Unresolved { site: self.placed.site(), name: AsRef::<Name>::as_ref(self.placed.entity()).clone() })
    }
}

impl Route {
    fn reaching(&self, owner: &Name, member: &Name) -> Option<&LawPath> {
        self.path.as_ref().filter(|path| path.member == *member).filter(|path| path.owner.admits(owner).is_some())
    }
}

impl LawOwner {
    fn admits<'name>(&self, owner: &'name Name) -> Option<&'name Name> {
        match self {
            LawOwner::Named(named) => Some(owner).filter(|candidate| *candidate == named),
            LawOwner::Operand => Some(owner),
        }
    }
}

impl Owners<'_> {
    fn route(self, written: &Written) -> Route {
        let path = written.operation().and_then(|operation| {
            let owner = self.owner(operation.owner(), written.fallback())?;
            Some(LawPath { owner, member: AsRef::<Name>::as_ref(operation).clone() })
        });
        Route { side: written.side(), path }
    }

    fn owner(self, owner: &OperationOwner, fallback: Option<&OperationOwner>) -> Option<LawOwner> {
        owner.typed().map_or(Some(LawOwner::Operand), |ty| self.typed(ty, fallback))
    }

    fn typed(self, ty: &TypeRef, fallback: Option<&OperationOwner>) -> Option<LawOwner> {
        let named = ty.dereferenced().named()?;
        let resolved: &Resolved = named.as_ref();
        let declared = resolved.crate_item().and_then(|id| self.model.declared(id));
        let traited = declared
            .filter(|item| item.kind() == ItemKind::Trait)
            .map(|_| ())
            .or_else(|| resolved.std_trait().map(|_| ()));
        let name = declared
            .map(AsRef::<Name>::as_ref)
            .or_else(|| resolved.std().and_then(ItemPath::last))
            .or_else(|| named.last_spelled())?;
        traited.map_or_else(
            || Some(LawOwner::Named(name.clone())),
            |()| fallback.and_then(|found| self.owner(found, None)),
        )
    }
}

impl fmt::Display for LawPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.owner {
            LawOwner::Named(owner) => write!(formatter, "{owner}::{}", self.member),
            LawOwner::Operand => write!(formatter, "{} of the type on its side", self.member),
        }
    }
}

impl fmt::Display for Route {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.path {
            Some(path) => write!(formatter, "{} resolves to {path}", self.side),
            None => write!(formatter, "{} is a closure the lint cannot resolve to an operation", self.side),
        }
    }
}

impl<'file> AsRef<Site<'file>> for Law<'file> {
    fn as_ref(&self) -> &Site<'file> {
        &self.site
    }
}

impl<'file> AsRef<Site<'file>> for Unresolved<'file> {
    fn as_ref(&self) -> &Site<'file> {
        &self.site
    }
}

impl fmt::Display for Unresolved<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "law {}! checks a closure the lint cannot resolve to an operation", self.name)
    }
}

impl Finding for Unresolved<'_> {
    fn fix(&self) -> Fix {
        Fix::new("pass the operation path (Type::function), or a closure that calls it on its parameters")
    }
}
