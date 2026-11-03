mod type_scope;

use syn::{Block, FnArg, ItemImpl, ItemTrait, ReturnType, Signature, Type};

use body_lifts::{BodyReading, Branches, Propagation};
use lint_core::Site;
use lint_layout::Name;
use lint_placement::CrateName;

use crate::endpoint::{Endpoint, Nature, Unnamed};
use crate::filler::Filler;
use crate::ground::Ground;
use crate::mutation::Mutation;
use crate::transition::Transition;
use type_scope::TypeScope;

#[derive(Debug)]
pub(super) struct SignatureReading {
    krate: CrateName,
    owner: Option<Name>,
    scope: TypeScope,
    ground: Ground,
    filler: Filler,
}

impl SignatureReading {
    pub(super) fn free(krate: CrateName, ground: Ground) -> SignatureReading {
        let scope = TypeScope::new(Endpoint::from(Unnamed::Opaque));
        SignatureReading { krate, owner: None, scope, ground, filler: Filler::Free }
    }

    pub(super) fn of_impl(krate: CrateName, found: &ItemImpl, ground: Ground) -> SignatureReading {
        let selfish =
            TypeScope::new(Endpoint::from(Unnamed::Opaque)).extended(&found.generics).endpoint(&found.self_ty);
        let filler = found
            .trait_
            .as_ref()
            .and_then(|(_, path, _)| path.segments.last())
            .map_or(Filler::Inherent, |segment| Filler::Implemented(Name::from(&segment.ident)));
        SignatureReading {
            krate,
            owner: Some(AsRef::<Name>::as_ref(&selfish).clone()),
            scope: TypeScope::new(selfish).extended(&found.generics),
            ground,
            filler,
        }
    }

    pub(super) fn of_trait(krate: CrateName, found: &ItemTrait, ground: Ground) -> SignatureReading {
        let name = Name::from(&found.ident);
        SignatureReading {
            krate,
            owner: Some(name.clone()),
            scope: TypeScope::new(Endpoint::new(Nature::Interface, name.clone())).extended(&found.generics),
            ground,
            filler: Filler::Provided(name),
        }
    }

    pub(super) fn transition<'file>(
        &self,
        site: Site<'file>,
        signature: &Signature,
        body: &Block,
    ) -> Transition<'file> {
        let scope = self.scope.extended(&signature.generics);
        let takes = signature
            .inputs
            .iter()
            .map(|input| match input {
                FnArg::Receiver(receiver) => scope.endpoint(&receiver.ty),
                FnArg::Typed(typed) => scope.endpoint(&typed.ty),
            })
            .collect();
        let gives = match &signature.output {
            ReturnType::Default => Endpoint::from(Unnamed::Unit),
            ReturnType::Type(_, returned) => scope.endpoint(returned),
        };
        Transition::new(self.krate.clone(), site, self.owner.clone(), Name::from(&signature.ident))
            .taking(takes, gives, Self::mutation(signature))
            .placed(self.ground, self.filler.clone(), BodyReading::classified(body))
            .branching(Branches::scored(body, Propagation::Ambient))
    }

    fn mutation(signature: &Signature) -> Mutation {
        let mutable = |found: &Type| matches!(found, Type::Reference(reference) if reference.mutability.is_some());
        let receiver = signature.receiver().filter(|receiver| mutable(&receiver.ty)).map(|_| Mutation::ThroughSelf);
        let parameter = signature
            .inputs
            .iter()
            .find(|input| matches!(input, FnArg::Typed(typed) if mutable(&typed.ty)))
            .map(|_| Mutation::ThroughParameter);
        receiver.or(parameter).unwrap_or(Mutation::Absent)
    }
}
