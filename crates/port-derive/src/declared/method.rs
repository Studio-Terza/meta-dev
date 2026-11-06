mod parameter;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{FnArg, Ident, ReturnType, Signature, Type};

use super::boundary::Spelling;
use super::names::OpsName;
pub use parameter::Parameter;

pub struct Method {
    name: Ident,
    parameters: Vec<Parameter>,
    output: ReturnType,
}

impl Method {
    pub fn declared(signature: &Signature) -> Method {
        let parameters = signature
            .inputs
            .iter()
            .filter_map(Self::typed)
            .enumerate()
            .map(|(position, tree)| Parameter::named(Spelling::input(position), tree))
            .collect();
        Method { name: signature.ident.clone(), parameters, output: signature.output.clone() }
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

    pub fn parameters(&self) -> &[Parameter] {
        &self.parameters
    }

    pub fn output(&self) -> &ReturnType {
        &self.output
    }

    pub fn pass(&self, ops: &OpsName) -> TokenStream {
        let op = Spelling::camel(&self.name);
        quote! { ::port_core::Pass<'_, #ops::#op> }
    }
}
