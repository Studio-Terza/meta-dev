use std::fmt;

use quote::ToTokens;
use syn::{FnArg, ImplItemFn, ItemImpl, ReturnType};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Shape {
    self_type: String,
    generics: String,
    platform: String,
    inputs: String,
    output: String,
}

impl Shape {
    pub fn describe(item: &ItemImpl, function: &ImplItemFn) -> Shape {
        let signature = &function.sig;
        let platform: Vec<String> =
            function.attrs.iter().filter(|attribute| attribute.path().is_ident("cfg")).map(Self::squeezed).collect();
        let inputs: Vec<String> = signature
            .inputs
            .iter()
            .map(|argument| match argument {
                FnArg::Receiver(receiver) => Self::squeezed(receiver),
                FnArg::Typed(typed) => Self::squeezed(&typed.ty),
            })
            .collect();
        Shape {
            self_type: Self::squeezed(&item.self_ty),
            generics: Self::squeezed(&signature.generics),
            platform: platform.join(", "),
            inputs: inputs.join(", "),
            output: match &signature.output {
                ReturnType::Default => String::new(),
                ReturnType::Type(_, ty) => Self::squeezed(ty),
            },
        }
    }

    fn squeezed(tokens: &impl ToTokens) -> String {
        tokens.to_token_stream().to_string().chars().filter(|letter| *letter != ' ').collect()
    }
}

impl fmt::Display for Shape {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}::{}({})", self.self_type, self.generics, self.inputs)?;
        if self.output.is_empty() { Ok(()) } else { write!(formatter, " -> {}", self.output) }
    }
}
