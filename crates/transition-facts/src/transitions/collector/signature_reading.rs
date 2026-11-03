mod type_scope;

use syn::{Block, FnArg, ItemImpl, ItemTrait, ReturnType, Signature, Type};

use body_lifts::{BodyReading, Branches, Propagation};
use lint_core::Site;
use lint_layout::Name;
use lint_placement::CrateName;

use crate::context::Context;
use crate::endpoint::{Endpoint, Nature, Unnamed};
use crate::filler::Filler;
use crate::function::Function;
use crate::ground::Ground;
use crate::mutation::Mutation;
use crate::resolved::Resolved;
use type_scope::TypeScope;

#[derive(Debug)]
pub(super) struct SignatureReading<'file> {
    context: Context<'file>,
    scope: TypeScope<'static>,
}

impl<'file> SignatureReading<'file> {
    pub(super) fn free(krate: &'file CrateName, ground: Ground) -> SignatureReading<'file> {
        let selfish = Endpoint::from(Unnamed::Opaque);
        let scope = TypeScope::root(selfish.nature());
        SignatureReading { context: Context::new(krate, selfish, ground, Filler::Free), scope }
    }

    pub(super) fn of_impl(krate: &'file CrateName, found: &ItemImpl, ground: Ground) -> SignatureReading<'file> {
        let outer = Endpoint::from(Unnamed::Opaque).nature();
        let selfish = TypeScope::declaring(outer, None, &found.generics).endpoint(&found.self_ty).or_opaque();
        let filler = found
            .trait_
            .as_ref()
            .and_then(|(_, path, _)| path.segments.last())
            .map_or(Filler::Inherent, |segment| Filler::Implemented(Name::from(&segment.ident)));
        let scope = TypeScope::declaring(selfish.nature(), None, &found.generics);
        SignatureReading { context: Context::new(krate, selfish, ground, filler), scope }
    }

    pub(super) fn of_trait(krate: &'file CrateName, found: &ItemTrait, ground: Ground) -> SignatureReading<'file> {
        let selfish = Endpoint::new(Nature::Interface, Name::from(&found.ident));
        let scope = TypeScope::declaring(selfish.nature(), None, &found.generics);
        SignatureReading { context: Context::new(krate, selfish, ground, Filler::Provided), scope }
    }

    pub(super) fn function(&self, site: Site<'file>, signature: &Signature, body: &Block) -> Function<'file> {
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
            ReturnType::Default => Resolved::Found(Endpoint::from(Unnamed::Unit)),
            ReturnType::Type(_, returned) => scope.endpoint(returned),
        };
        Function::new(site, Name::from(&signature.ident), takes, gives).shaped(
            Self::mutation(signature),
            BodyReading::classified(body),
            Branches::scored(body, Propagation::Ambient),
        )
    }

    pub(super) fn into_context(self) -> Context<'file> {
        self.context
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
