use syn::{Generics, Ident, PredicateType, Type, TypeParamBound, WherePredicate};

use lint_layout::Name;

#[derive(Debug)]
pub(super) struct Constraint<'generics> {
    name: &'generics Ident,
    bound: &'generics TypeParamBound,
}

impl<'generics> Constraint<'generics> {
    pub(super) fn declared(generics: &'generics Generics) -> Vec<Constraint<'generics>> {
        generics
            .type_params()
            .flat_map(|parameter| parameter.bounds.iter().map(|bound| Constraint { name: &parameter.ident, bound }))
            .chain(generics.where_clause.iter().flat_map(|clause| clause.predicates.iter()).flat_map(Self::required))
            .collect()
    }

    pub(super) fn bounds<'all>(
        constraints: &'all [Constraint<'generics>],
        name: &'all Name,
    ) -> impl Iterator<Item = &'generics TypeParamBound> + 'all {
        constraints.iter().filter(move |constraint| *constraint.name == *name).map(|constraint| constraint.bound)
    }

    fn required(predicate: &'generics WherePredicate) -> Vec<Constraint<'generics>> {
        match predicate {
            WherePredicate::Type(typed) => Self::typed(typed),
            _ => Vec::new(),
        }
    }

    fn typed(typed: &'generics PredicateType) -> Vec<Constraint<'generics>> {
        Self::ident_of(&typed.bounded_ty)
            .map(|name| typed.bounds.iter().map(|bound| Constraint { name, bound }).collect())
            .unwrap_or_default()
    }

    fn ident_of(bounded: &Type) -> Option<&Ident> {
        match bounded {
            Type::Path(path) => path.path.get_ident(),
            _ => None,
        }
    }
}
