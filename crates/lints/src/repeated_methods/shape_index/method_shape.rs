use std::fmt;

use proc_macro2::{Group, Ident, TokenStream, TokenTree};
use quote::ToTokens;
use syn::{FnArg, ReturnType, Signature};

use lint_core::Name;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MethodShape {
    name: Name,
    signature: String,
}

impl MethodShape {
    const OWN_TYPE: &str = "Self";

    #[must_use]
    pub fn described(signature: &Signature, self_type: &Name) -> MethodShape {
        let erased = |tokens: TokenStream| Self::erased(tokens, self_type).to_string();
        let inputs: Vec<String> = signature
            .inputs
            .iter()
            .map(|argument| match argument {
                FnArg::Receiver(receiver) => erased(receiver.to_token_stream()),
                FnArg::Typed(typed) => erased(typed.ty.to_token_stream()),
            })
            .collect();
        let output = match &signature.output {
            ReturnType::Default => String::new(),
            ReturnType::Type(arrow, ty) => {
                format!(" {} {}", arrow.to_token_stream(), erased(ty.to_token_stream()))
            }
        };
        MethodShape {
            name: Name::from(&signature.ident),
            signature: format!("{}({}){output}", erased(signature.generics.to_token_stream()), inputs.join(", ")),
        }
    }

    fn erased(tokens: TokenStream, self_type: &Name) -> TokenStream {
        tokens
            .into_iter()
            .map(|token| match token {
                TokenTree::Ident(ident) if *self_type == ident => {
                    TokenTree::Ident(Ident::new(Self::OWN_TYPE, ident.span()))
                }
                TokenTree::Group(group) => {
                    TokenTree::Group(Group::new(group.delimiter(), Self::erased(group.stream(), self_type)))
                }
                TokenTree::Ident(_) | TokenTree::Punct(_) | TokenTree::Literal(_) => token,
            })
            .collect()
    }
}

impl fmt::Display for MethodShape {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "fn {}{}", self.name, self.signature)
    }
}
