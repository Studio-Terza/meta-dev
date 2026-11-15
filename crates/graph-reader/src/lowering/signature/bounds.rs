use syn::{GenericParam, Generics, Type, TypeParamBound, WherePredicate};

use lint_layout::Name;

use crate::namespace::UsePath;

#[derive(Clone, Debug)]
pub(crate) struct ParamBounds {
    param: Name,
    traits: Vec<UsePath>,
}

impl ParamBounds {
    pub(crate) fn declared(generics: &Generics) -> Vec<ParamBounds> {
        let declared = generics.params.iter().filter_map(Self::of_param);
        let predicated =
            generics.where_clause.iter().flat_map(|clause| clause.predicates.iter()).filter_map(Self::of_predicate);
        declared.chain(predicated).collect()
    }

    pub(crate) fn param(&self) -> &Name {
        &self.param
    }

    pub(crate) fn traits(&self) -> &[UsePath] {
        &self.traits
    }

    fn of_param(parameter: &GenericParam) -> Option<ParamBounds> {
        match parameter {
            GenericParam::Type(declared) => Some(ParamBounds {
                param: Name::from(&declared.ident),
                traits: declared.bounds.iter().filter_map(Self::trait_path).collect(),
            }),
            GenericParam::Lifetime(_) | GenericParam::Const(_) => None,
        }
    }

    fn of_predicate(predicate: &WherePredicate) -> Option<ParamBounds> {
        match predicate {
            WherePredicate::Type(typed) => Self::param_name(&typed.bounded_ty)
                .map(|param| ParamBounds { param, traits: typed.bounds.iter().filter_map(Self::trait_path).collect() }),
            _ => None,
        }
    }

    fn param_name(ty: &Type) -> Option<Name> {
        match ty {
            Type::Path(path) => path.path.get_ident().map(Name::from),
            _ => None,
        }
    }

    pub(crate) fn trait_path(bound: &TypeParamBound) -> Option<UsePath> {
        match bound {
            TypeParamBound::Trait(traited) => Some(UsePath::from(&traited.path)),
            _ => None,
        }
    }
}
