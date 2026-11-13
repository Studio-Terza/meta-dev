use std::collections::BTreeMap;

use syn::{Type, TypePath};

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

    pub fn kind(&self, file: &SourceFile, module: &ModulePath, implemented: &Type) -> Kind {
        match implemented {
            Type::Path(path) => path.path.segments.last().map_or(Kind::Other, |last| {
                self.kind_named(file.head().placement().crate_root(), module, path, Name::from(&last.ident))
            }),
            _ => Kind::Other,
        }
    }

    fn kind_named(&self, crate_root: &CrateRoot, module: &ModulePath, path: &TypePath, name: Name) -> Kind {
        let named = Named { crate_root, name };
        let local = Some(path)
            .filter(|candidate| candidate.qself.is_none())
            .filter(|unqualified| unqualified.path.segments.len() == 1)
            .and_then(|_| self.placed.get(&Placed { named: named.clone(), module: module.clone() }).copied());
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
