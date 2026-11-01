mod erasure;

use std::fmt;

use quote::ToTokens;
use syn::{FnArg, ReturnType, Signature};

use lint_layout::Name;

pub use erasure::Erasure;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MethodShape {
    name: Name,
    signature: String,
}

impl MethodShape {
    #[must_use]
    pub fn described(signature: &Signature, erasure: &Erasure) -> MethodShape {
        let inputs: Vec<String> = signature
            .inputs
            .iter()
            .map(|argument| match argument {
                FnArg::Receiver(receiver) => erasure.spelled(receiver).to_string(),
                FnArg::Typed(typed) => erasure.spelled_input(&typed.ty).to_string(),
            })
            .collect();
        let output = match &signature.output {
            ReturnType::Default => String::new(),
            ReturnType::Type(arrow, ty) => format!(" {} {}", arrow.to_token_stream(), erasure.spelled(ty)),
        };
        MethodShape {
            name: Name::from(&signature.ident),
            signature: format!("{}({}){output}", erasure.spelled(&signature.generics), inputs.join(", ")),
        }
    }
}

impl fmt::Display for MethodShape {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "fn {}{}", self.name, self.signature)
    }
}
