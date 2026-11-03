use syn::{Generics, Ident, Type, TypeParamBound, WherePredicate};

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
        constraints.iter().filter(move |constraint| *name == *constraint.name).map(|constraint| constraint.bound)
    }

    fn required(predicate: &'generics WherePredicate) -> Vec<Constraint<'generics>> {
        let typed = match predicate {
            WherePredicate::Type(typed) => Some(typed),
            _ => None,
        };
        let named = typed.and_then(|typed| match &typed.bounded_ty {
            Type::Path(path) => path.path.get_ident(),
            _ => None,
        });
        typed
            .zip(named)
            .map(|(typed, name)| typed.bounds.iter().map(|bound| Constraint { name, bound }).collect())
            .unwrap_or_default()
    }
}
