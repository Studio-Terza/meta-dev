use std::marker::PhantomData;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::Ident;

use super::boundary::{Derivation, Spelling};

pub trait Role {
    fn spelled(caller: &Ident) -> Ident;
}

#[derive(Debug)]
pub struct CallerRole;

#[derive(Debug)]
pub struct FillerRole;

#[derive(Debug)]
pub struct OpsRole;

#[derive(Debug)]
pub struct Name<Of: Role> {
    ident: Ident,
    role: PhantomData<Of>,
}

pub type CallerName = Name<CallerRole>;
pub type FillerName = Name<FillerRole>;
pub type OpsName = Name<OpsRole>;

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
