use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Block, Error, FnArg, GenericParam, Ident, Receiver, Signature};

pub struct Refusal;

impl Refusal {
    pub fn qualifiers(signature: &Signature) -> syn::Result<()> {
        signature
            .asyncness
            .map(|token| token.to_token_stream())
            .or(signature.constness.map(|token| token.to_token_stream()))
            .or(signature.unsafety.map(|token| token.to_token_stream()))
            .or(signature.abi.as_ref().map(ToTokens::to_token_stream))
            .map_or(Ok(()), |token| {
                Err(Error::new_spanned(token, "async, const, unsafe and extern port methods are not supported"))
            })
    }

    pub fn generics(signature: &Signature, per_query: Option<&Ident>) -> syn::Result<()> {
        let declared: Vec<Option<&Ident>> = signature.generics.params.iter().map(Self::type_name).collect();
        let wanted: Vec<Option<&Ident>> = per_query.into_iter().map(Some).collect();
        (declared == wanted).then_some(()).ok_or_else(|| {
            Error::new_spanned(
                &signature.generics,
                "port methods take no generics; a per-query op is written #[op(Q)] with exactly one type parameter Q",
            )
        })
    }

    pub fn receiver(inputs: &Punctuated<FnArg, Comma>) -> syn::Result<()> {
        inputs
            .first()
            .and_then(Self::receiver_of)
            .filter(|receiver| receiver.mutability.is_none())
            .filter(|receiver| receiver.colon_token.is_none())
            .filter(|receiver| receiver.reference.as_ref().is_some_and(|(_, lifetime)| lifetime.is_none()))
            .map(|_| ())
            .ok_or_else(|| Error::new_spanned(inputs, "port methods take &self and nothing else as receiver"))
    }

    pub fn provided_query(
        signature: &Signature,
        per_query: Option<&Ident>,
        provided: Option<&Block>,
    ) -> syn::Result<()> {
        per_query.zip(provided).map_or(Ok(()), |_| {
            Err(Error::new_spanned(signature, "#[op(Q)] belongs on a required method, not a provided one"))
        })
    }

    fn type_name(parameter: &GenericParam) -> Option<&Ident> {
        match parameter {
            GenericParam::Type(declared) => Some(&declared.ident),
            GenericParam::Lifetime(_) | GenericParam::Const(_) => None,
        }
    }

    fn receiver_of(argument: &FnArg) -> Option<&Receiver> {
        match argument {
            FnArg::Receiver(receiver) => Some(receiver),
            FnArg::Typed(_) => None,
        }
    }
}
