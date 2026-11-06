mod parameter;
mod refusal;
mod returned;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Block, Error, Ident, Signature, TraitItemFn};

use super::names::OpsName;
use crate::boundary::Spelling;
pub use parameter::Parameter;
use refusal::Refusal;
pub use returned::Returned;

struct Attributes {
    per_query: Option<Ident>,
    kept: Vec<Attribute>,
}

impl Attributes {
    fn split(attrs: Vec<Attribute>) -> syn::Result<Attributes> {
        let (operations, kept): (Vec<_>, Vec<_>) =
            attrs.into_iter().partition(|attribute| attribute.path().is_ident("op"));
        Self::queried(&operations).map(|per_query| Attributes { per_query, kept })
    }

    fn queried(operations: &[Attribute]) -> syn::Result<Option<Ident>> {
        operations.get(1).map_or_else(
            || operations.first().map(Attribute::parse_args::<Ident>).transpose(),
            |second| Err(Error::new_spanned(second, "a method takes at most one #[op(Q)]")),
        )
    }
}

pub struct Method {
    attributes: Vec<Attribute>,
    signature: Signature,
    provided: Option<Block>,
    per_query: Option<Ident>,
    parameters: Vec<Parameter>,
    returned: Returned,
}

impl Method {
    pub fn declared(item: TraitItemFn) -> syn::Result<Method> {
        let TraitItemFn { attrs, sig, default, .. } = item;
        let Attributes { per_query, kept } = Attributes::split(attrs)?;
        Refusal::qualifiers(&sig)?;
        Refusal::generics(&sig, per_query.as_ref())?;
        Refusal::receiver(&sig.inputs)?;
        Refusal::provided_query(&sig, per_query.as_ref(), default.as_ref())?;
        let parameters = sig.inputs.iter().skip(1).map(Parameter::parsed).collect::<syn::Result<Vec<_>>>()?;
        let returned = Returned::classified(&sig.output)?;
        Ok(Method { attributes: kept, signature: sig, provided: default, per_query, parameters, returned })
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

    pub fn returned(&self) -> &Returned {
        &self.returned
    }

    pub fn counting(&self) -> Ident {
        self.returned.counting(self.signature.ident.span())
    }

    pub fn op_struct(&self) -> Option<Ident> {
        self.per_query.is_none().then(|| Spelling::camel(&self.signature.ident))
    }

    pub fn pass(&self, ops: &OpsName) -> TokenStream {
        let own = self.op_struct().map(|op| quote! { ::port_core::Pass<'_, #ops::#op> });
        self.per_query.as_ref().map(|query| quote! { ::port_core::Pass<'_, #query> }).or(own).unwrap_or_default()
    }
}
