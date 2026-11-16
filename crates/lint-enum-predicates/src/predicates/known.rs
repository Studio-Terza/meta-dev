use std::collections::BTreeMap;

use syntax::{Type, TypePath};

use lint_core::SourceFile;
use lint_layout::Name;
use lint_placement::{CrateRoot, ModulePath};

use super::Kind;
use super::declaration::Declaration;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Named<'head> {
    crate_root: &'head CrateRoot,
    name: Name,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Placed<'head> {
    named: Named<'head>,
    module: ModulePath,
}

#[derive(Debug)]
pub struct Target {
    name: Name,
    local: Option<ModulePath>,
}

impl Target {
    pub fn implemented(module: &ModulePath, implemented: &Type) -> Option<Target> {
        match implemented {
            Type::Path(path) => Self::pathed(module, path),
            _ => None,
        }
    }

    fn pathed(module: &ModulePath, path: &TypePath) -> Option<Target> {
        let name = path.path.segments.last().map(|last| Name::from(&last.ident))?;
        let single = Some(path)
            .filter(|candidate| candidate.qself.is_none())
            .filter(|unqualified| unqualified.path.segments.len() == 1);
        Some(Target { name, local: single.map(|_| module.clone()) })
    }
}

#[derive(Debug, Default)]
pub struct Known<'head> {
    named: BTreeMap<Named<'head>, Vec<Kind>>,
    placed: BTreeMap<Placed<'head>, Kind>,
}

impl<'head> Known<'head> {
    pub fn recorded(
        mut self,
        crate_root: &'head CrateRoot,
        module: &ModulePath,
        name: &Name,
        kind: Kind,
    ) -> Known<'head> {
        let named = Named { crate_root, name: name.clone() };
        self.named.entry(named.clone()).or_default().push(kind);
        self.placed.entry(Placed { named, module: module.clone() }).or_insert(kind);
        self
    }

    pub fn kind(&self, file: &SourceFile, target: &Target) -> Kind {
        let named = Named { crate_root: file.head().placement().crate_root(), name: target.name.clone() };
        let local = target
            .local
            .as_ref()
            .and_then(|module| self.placed.get(&Placed { named: named.clone(), module: module.clone() }).copied());
        let candidates = self.named.get(&named).map_or(&[][..], Vec::as_slice);
        let enums =
            candidates.first().filter(|_| candidates.iter().all(|kind| *kind == Kind::Enum)).map(|_| Kind::Enum);
        local.or(enums).unwrap_or(Kind::Other)
    }
}

impl<'head> FromIterator<Vec<Declaration<'head>>> for Known<'head> {
    fn from_iter<Parts: IntoIterator<Item = Vec<Declaration<'head>>>>(parts: Parts) -> Known<'head> {
        parts.into_iter().flatten().fold(Known::default(), |known, found| found.filed(known))
    }
}
