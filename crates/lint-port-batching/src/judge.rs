use std::collections::BTreeSet;

use syn::{GenericParam, Generics, Type, TypeParamBound, WherePredicate};

use lint_core::Syntax;
use lint_fakes_at_ports::{Kind, PortTraits};
use lint_layout::Name;
use lint_paths::{Imports, Module};

use crate::spelling::Spelling;

#[derive(Debug)]
pub struct Judge<'run> {
    imports: &'run Imports<'run>,
    ports: &'run PortTraits,
}

impl<'run> Judge<'run> {
    pub const fn new(imports: &'run Imports<'run>, ports: &'run PortTraits) -> Self {
        Judge { imports, ports }
    }

    pub const fn ports(&self) -> &PortTraits {
        self.ports
    }

    pub fn generics(&self, module: &Module<'_>, generics: &Generics) -> BTreeSet<Name> {
        let params = generics.params.iter().filter_map(|param| self.param(module, param));
        let predicates = generics
            .where_clause
            .iter()
            .flat_map(|clause| &clause.predicates)
            .filter_map(|predicate| self.predicate(module, predicate));
        params.chain(predicates).collect()
    }

    pub fn port_type(&self, module: &Module<'_>, generics: &BTreeSet<Name>, ty: &Type) -> Option<Name> {
        match Spelling::inner(ty) {
            Some(inner) => self.port_type(module, generics, inner),
            None => self.bounded(module, ty).or_else(|| Spelling::generic(ty, generics)),
        }
    }

    fn bounded(&self, module: &Module<'_>, ty: &Type) -> Option<Name> {
        Syntax::bounds(ty).into_iter().flatten().find_map(|bound| self.bound(module, bound))
    }

    fn param(&self, module: &Module<'_>, param: &GenericParam) -> Option<Name> {
        match param {
            GenericParam::Type(param) => {
                param.bounds.iter().find_map(|bound| self.bound(module, bound)).map(|_| Name::from(&param.ident))
            }
            GenericParam::Lifetime(_) | GenericParam::Const(_) => None,
        }
    }

    fn predicate(&self, module: &Module<'_>, predicate: &WherePredicate) -> Option<Name> {
        match predicate {
            WherePredicate::Type(predicate) => predicate
                .bounds
                .iter()
                .find_map(|bound| self.bound(module, bound))
                .and(Spelling::ident(&predicate.bounded_ty))
                .map(Name::from),
            _ => None,
        }
    }

    fn bound(&self, module: &Module<'_>, bound: &TypeParamBound) -> Option<Name> {
        Syntax::trait_bound(bound)
            .filter(|traited| self.ports.judged(self.imports, module, &traited.path) == Kind::Port)
            .and_then(|traited| traited.path.segments.last())
            .map(|last| Name::from(&last.ident))
    }
}
