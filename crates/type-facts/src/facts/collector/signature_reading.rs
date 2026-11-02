mod type_scope;

use std::collections::BTreeSet;

use syn::{FnArg, GenericParam, Generics, ReturnType, Signature};

use lint_core::Site;
use lint_layout::Name;
use lint_placement::CrateName;

use crate::endpoint::Endpoint;
use crate::transition::Transition;
use type_scope::TypeScope;

#[derive(Debug)]
pub(super) struct SignatureReading<'owner> {
    owner: Option<&'owner Name>,
    generics: BTreeSet<Name>,
}

impl<'owner> SignatureReading<'owner> {
    pub(super) fn within(owner: Option<&'owner Name>, outer: &Generics) -> SignatureReading<'owner> {
        SignatureReading { owner, generics: Self::parameters(outer) }
    }

    pub(super) fn transition(&self, krate: CrateName, site: Site, signature: &Signature) -> Transition {
        let scope =
            TypeScope::new(self.owner, self.generics.union(&Self::parameters(&signature.generics)).cloned().collect());
        let takes = signature
            .inputs
            .iter()
            .map(|input| match input {
                FnArg::Receiver(_) => scope.receiver(),
                FnArg::Typed(typed) => scope.endpoint(&typed.ty),
            })
            .collect();
        let gives = match &signature.output {
            ReturnType::Default => Endpoint::Unit,
            ReturnType::Type(_, returned) => scope.endpoint(returned),
        };
        Transition::new(krate, site, self.owner.cloned(), Name::from(&signature.ident)).taking(takes, gives)
    }

    fn parameters(generics: &Generics) -> BTreeSet<Name> {
        generics
            .params
            .iter()
            .filter_map(|parameter| match parameter {
                GenericParam::Type(found) => Some(Name::from(&found.ident)),
                _ => None,
            })
            .collect()
    }
}
