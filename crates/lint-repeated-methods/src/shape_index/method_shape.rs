mod erasure;

use std::fmt;

use syn::Signature;

use lint_core::Syntax;

use lint_layout::Name;

pub use erasure::Erasure;
use newtype::{Arbitrary, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Spelling(String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MethodShape {
    name: Name,
    signature: Spelling,
}

impl MethodShape {
    #[must_use]
    pub fn described(signature: &Signature, erasure: &Erasure<'_>) -> MethodShape {
        let inputs: Vec<String> =
            signature.inputs.iter().map(|argument| erasure.spelled_argument(argument).to_string()).collect();
        let output =
            Syntax::returned(&signature.output).map_or_else(String::new, |ty| format!(" -> {}", erasure.spelled(ty)));
        MethodShape {
            name: Name::from(&signature.ident),
            signature: Spelling::new(format!(
                "{}({}){output}",
                erasure.spelled(&signature.generics),
                inputs.join(", ")
            )),
        }
    }
}

impl fmt::Display for MethodShape {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "fn {}{}", self.name, self.signature)
    }
}
