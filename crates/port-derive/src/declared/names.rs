use std::marker::PhantomData;

use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;

use super::boundary::{Derivation, Spelling};

pub(crate) trait Role {
    fn spelled(caller: &Ident) -> Ident;
}

#[derive(Debug)]
pub(crate) struct CallerRole;

#[derive(Debug)]
pub(crate) struct FillerRole;

#[derive(Debug)]
pub(crate) struct OpsRole;

#[derive(Debug)]
pub(crate) struct Name<Of: Role> {
    ident: Ident,
    role: PhantomData<Of>,
}

pub(crate) type CallerName = Name<CallerRole>;
pub(crate) type FillerName = Name<FillerRole>;
pub(crate) type OpsName = Name<OpsRole>;

impl Role for CallerRole {
    fn spelled(caller: &Ident) -> Ident {
        caller.clone()
    }
}

impl Role for FillerRole {
    fn spelled(caller: &Ident) -> Ident {
        Spelling::derived(caller, Derivation::Filler)
    }
}

impl Role for OpsRole {
    fn spelled(caller: &Ident) -> Ident {
        Spelling::derived(caller, Derivation::OpsModule)
    }
}

impl<Of: Role> From<&Ident> for Name<Of> {
    fn from(caller: &Ident) -> Self {
        Name { ident: Of::spelled(caller), role: PhantomData }
    }
}

impl<Of: Role> ToTokens for Name<Of> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
    }
}
