use std::fmt;

use quote::ToTokens;
use syn::{FnArg, ImplItemFn, ItemImpl};

use lint_core::Syntax;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Squeezed(String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Shape {
    self_type: Squeezed,
    generics: Squeezed,
    platform: Squeezed,
    inputs: Squeezed,
    output: Option<Squeezed>,
}

impl Shape {
    pub fn describe(item: &ItemImpl, function: &ImplItemFn) -> Shape {
        let signature = &function.sig;
        let platform: Vec<Squeezed> =
            function.attrs.iter().filter(|attribute| attribute.path().is_ident("cfg")).map(Squeezed::from).collect();
        let inputs: Vec<Squeezed> = signature.inputs.iter().map(Squeezed::of_argument).collect();
        Shape {
            self_type: Squeezed::from(&item.self_ty),
            generics: Squeezed::from(&signature.generics),
            platform: Squeezed::joined(platform),
            inputs: Squeezed::joined(inputs),
            output: Syntax::returned(&signature.output).map(Squeezed::from),
        }
    }
}

impl Squeezed {
    fn of_argument(argument: &FnArg) -> Squeezed {
        match argument {
            FnArg::Receiver(receiver) => Squeezed::from(receiver),
            FnArg::Typed(typed) => Squeezed::from(&typed.ty),
        }
    }

    fn joined(parts: Vec<Squeezed>) -> Squeezed {
        Squeezed(parts.into_iter().map(|part| part.0).collect::<Vec<_>>().join(", "))
    }
}

impl<Tokens: ToTokens> From<&Tokens> for Squeezed {
    fn from(tokens: &Tokens) -> Squeezed {
        Squeezed(tokens.to_token_stream().to_string().chars().filter(|letter| *letter != ' ').collect())
    }
}

impl fmt::Display for Squeezed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl fmt::Display for Shape {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}::{}({})", self.self_type, self.generics, self.inputs)?;
        self.output.as_ref().map_or(Ok(()), |output| write!(formatter, " -> {output}"))
    }
}
