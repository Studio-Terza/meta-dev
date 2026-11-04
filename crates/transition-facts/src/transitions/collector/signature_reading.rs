mod calls;
mod type_scope;

use syn::{Block, FnArg, ImplItem, ItemImpl, ItemTrait, ReturnType, Signature, Type};

use body_lifts::{BodyReading, Branches, Propagation};
use lint_core::Site;
use lint_layout::{Name, Whole, Word};

use super::origins::Origins;
use crate::context::Context;
use crate::endpoint::{Endpoint, Nature, Unnamed};
use crate::filler::Filler;
use crate::function::Function;
use crate::ground::Ground;
use crate::mutation::Mutation;
use crate::resolved::Resolved;
use crate::wrapping::Wrapping;
use calls::Calls;
use type_scope::TypeScope;

#[derive(Debug)]
pub(super) struct SignatureReading<'origins, 'file> {
    context: Context<'file>,
    scope: TypeScope<'static>,
    origins: &'origins Origins<'file>,
    output: Option<Wrapping>,
}

impl<'origins, 'file> SignatureReading<'origins, 'file> {
    const OUTPUT: Word<'static> = Word::new("Output");

    pub(super) fn free(origins: &'origins Origins<'file>, ground: Ground) -> SignatureReading<'origins, 'file> {
        let selfish = Endpoint::from(Unnamed::Opaque);
        let scope = TypeScope::root(selfish.nature());
        SignatureReading {
            context: Context::new(origins.own(), selfish, ground, Filler::Free),
            scope,
            origins,
            output: None,
        }
    }

    pub(super) fn of_impl(
        origins: &'origins Origins<'file>,
        found: &ItemImpl,
        ground: Ground,
    ) -> SignatureReading<'origins, 'file> {
        let outer = Endpoint::from(Unnamed::Opaque).nature();
        let selfish =
            origins.endpoint(TypeScope::declaring(outer, None, &found.generics).endpoint(&found.self_ty).or_opaque());
        let filler = found
            .trait_
            .as_ref()
            .and_then(|(_, path, _)| path.segments.last())
            .map_or(Filler::Inherent, |segment| Filler::Implemented(Name::from(&segment.ident)));
        let declared = TypeScope::declaring(selfish.nature(), None, &found.generics);
        let scope = found.items.iter().filter_map(Self::output).fold(declared, |scope, output| {
            let resolved = scope.endpoint(output);
            scope.associating(resolved)
        });
        let output = found.items.iter().find_map(Self::output).map(Wrapping::peeled);
        SignatureReading { context: Context::new(origins.own(), selfish, ground, filler), scope, origins, output }
    }

    pub(super) fn of_trait(
        origins: &'origins Origins<'file>,
        found: &ItemTrait,
        ground: Ground,
    ) -> SignatureReading<'origins, 'file> {
        let selfish = Endpoint::new(Nature::Interface, Name::from(&found.ident));
        let scope = TypeScope::declaring(selfish.nature(), None, &found.generics);
        SignatureReading {
            context: Context::new(origins.own(), selfish, ground, Filler::Provided),
            scope,
            origins,
            output: None,
        }
    }

    pub(super) fn function(&self, site: Site<'file>, signature: &Signature, body: &Block) -> Function<'file> {
        let scope = self.scope.extended(&signature.generics);
        let takes = signature.inputs.iter().map(|input| self.taken(&scope, input)).collect();
        let gives = self.given(&scope, &signature.output);
        Function::new(site, Name::from(&signature.ident), takes, gives)
            .shaped(
                Self::mutation(signature),
                BodyReading::classified(body),
                Branches::scored(body, Propagation::Ambient),
            )
            .wrapped(
                Wrapping::returned(&signature.output, self.output),
                signature.inputs.iter().map(Wrapping::taken).collect(),
            )
            .called(Calls::made(signature, body))
    }

    pub(super) fn into_context(self) -> Context<'file> {
        self.context
    }

    fn output(item: &ImplItem) -> Option<&Type> {
        match item {
            ImplItem::Type(associated) => {
                Name::from(&associated.ident).matches::<Whole>(&[Self::OUTPUT]).map(|_| &associated.ty)
            }
            _ => None,
        }
    }

    fn taken(&self, scope: &TypeScope<'_>, input: &FnArg) -> Resolved {
        match input {
            FnArg::Receiver(receiver) => scope.endpoint(&receiver.ty),
            FnArg::Typed(typed) => self.origins.located(scope.endpoint(&typed.ty)),
        }
    }

    fn given(&self, scope: &TypeScope<'_>, output: &ReturnType) -> Resolved {
        match output {
            ReturnType::Default => Resolved::Found(Endpoint::from(Unnamed::Unit)),
            ReturnType::Type(_, returned) => self.origins.located(scope.endpoint(returned)),
        }
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
