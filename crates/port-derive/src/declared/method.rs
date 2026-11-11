mod parameter;
mod per_type;
mod returned;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{FnArg, Generics, Ident, ReturnType, TraitItemFn, Type};

use super::boundary::Spelling;
use super::names::OpsName;
pub use parameter::Parameter;
use per_type::PerType;
pub use returned::Returned;

pub struct Method {
    name: Ident,
    generics: Generics,
    parameters: Vec<Parameter>,
    output: ReturnType,
    returned: Returned,
    per_type: Option<TokenStream>,
}

impl Method {
    pub fn declared(function: &mut TraitItemFn) -> Method {
        let per_type = PerType::stripped(function);
        let signature = &function.sig;
        let parameters = signature
            .inputs
            .iter()
            .filter_map(Self::typed)
            .enumerate()
            .map(|(position, tree)| Parameter::named(Spelling::input(position), tree))
            .collect();
        Method {
            name: signature.ident.clone(),
            generics: signature.generics.clone(),
            parameters,
            output: signature.output.clone(),
            returned: Returned::parsed(&signature.output),
            per_type,
        }
    }

    fn typed(input: &FnArg) -> Option<&Type> {
        match input {
            FnArg::Typed(typed) => Some(&typed.ty),
            FnArg::Receiver(_) => None,
        }
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }

    pub fn generics(&self) -> &Generics {
        &self.generics
    }

    pub fn parameters(&self) -> &[Parameter] {
        &self.parameters
    }

    pub fn output(&self) -> &ReturnType {
        &self.output
    }

    pub fn returned(&self) -> &Returned {
        &self.returned
    }

    pub fn op_struct(&self) -> Option<Ident> {
        self.per_type.as_ref().map_or_else(|| Some(Spelling::camel(&self.name)), |_| None)
    }

    pub fn pass(&self, ops: &OpsName) -> TokenStream {
        let named = self.per_type.clone().unwrap_or_else(|| {
            let op = Spelling::camel(&self.name);
            quote!(#ops::#op)
        });
        quote! { ::port_core::Pass<'_, #named> }
    }
}
