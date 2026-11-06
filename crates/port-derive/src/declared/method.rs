mod parameter;
mod refusal;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Block, Ident, Signature, TraitItemFn};

use super::boundary::Spelling;
use super::names::OpsName;
pub use parameter::Parameter;
use refusal::Refusal;

pub struct Method {
    attributes: Vec<Attribute>,
    signature: Signature,
    provided: Option<Block>,
    parameters: Vec<Parameter>,
}

impl Method {
    pub fn declared(item: TraitItemFn) -> syn::Result<Method> {
        let TraitItemFn { attrs, sig, default, .. } = item;
        Refusal::qualifiers(&sig)?;
        Refusal::generics(&sig)?;
        Refusal::receiver(&sig.inputs)?;
        let parameters = sig.inputs.iter().skip(1).map(Parameter::parsed).collect::<syn::Result<Vec<_>>>()?;
        Ok(Method { attributes: attrs, signature: sig, provided: default, parameters })
    }

    pub fn attributes(&self) -> &[Attribute] {
        &self.attributes
    }

    pub fn signature(&self) -> &Signature {
        &self.signature
    }

    pub fn provided(&self) -> Option<&Block> {
        self.provided.as_ref()
    }

    pub fn parameters(&self) -> &[Parameter] {
        &self.parameters
    }

    pub fn op_struct(&self) -> Ident {
        Spelling::camel(&self.signature.ident)
    }

    pub fn pass(&self, ops: &OpsName) -> TokenStream {
        let op = self.op_struct();
        quote! { ::port_core::Pass<'_, #ops::#op> }
    }
}
