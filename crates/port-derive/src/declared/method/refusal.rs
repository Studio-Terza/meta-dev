use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Error, FnArg, Receiver, Signature};

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

    pub fn generics(signature: &Signature) -> syn::Result<()> {
        signature
            .generics
            .params
            .is_empty()
            .then_some(())
            .ok_or_else(|| Error::new_spanned(&signature.generics, "port methods take no generics"))
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

    fn receiver_of(argument: &FnArg) -> Option<&Receiver> {
        match argument {
            FnArg::Receiver(receiver) => Some(receiver),
            FnArg::Typed(_) => None,
        }
    }
}
