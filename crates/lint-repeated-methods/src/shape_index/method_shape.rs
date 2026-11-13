mod boundary;
mod erasure;

use std::fmt;

use syn::{FnArg, Signature};

use lint_core::{Site, Syntax};

use lint_layout::Name;

pub use boundary::Spelling;
pub use erasure::Erasure;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MethodShape {
    name: Name,
    signature: Spelling,
}

impl fmt::Display for MethodShape {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "fn {}{}", self.name, self.signature)
    }
}

#[derive(Debug)]
struct SpelledInput {
    written: Option<Spelling>,
    erased: Spelling,
}

impl SpelledInput {
    fn new(argument: &FnArg, erasure: &Erasure<'_>) -> SpelledInput {
        SpelledInput { written: Self::written(argument), erased: Spelling::from(erasure.spelled_argument(argument)) }
    }

    fn written(argument: &FnArg) -> Option<Spelling> {
        match argument {
            FnArg::Receiver(_) => None,
            FnArg::Typed(typed) => Some(Spelling::from(&*typed.ty)),
        }
    }

    fn spelled(&self, inner: Option<&Spelling>) -> Spelling {
        self.written
            .as_ref()
            .zip(inner)
            .filter(|(written, found)| written == found)
            .map_or_else(|| self.erased.clone(), |_| Spelling::inner())
    }
}

#[derive(Debug)]
pub struct SpelledMethod<'head> {
    name: Name,
    site: Site<'head>,
    generics: Spelling,
    inputs: Vec<SpelledInput>,
    output: Spelling,
}

impl<'head> SpelledMethod<'head> {
    pub fn written(signature: &Signature, erasure: &Erasure<'_>, site: Site<'head>) -> SpelledMethod<'head> {
        SpelledMethod {
            name: Name::from(&signature.ident),
            site,
            generics: Spelling::from(erasure.spelled(&signature.generics)),
            inputs: signature.inputs.iter().map(|argument| SpelledInput::new(argument, erasure)).collect(),
            output: Spelling::returning(Syntax::returned(&signature.output).map(|ty| erasure.spelled(ty))),
        }
    }

    pub fn shaped(&self, inner: Option<&Spelling>) -> MethodShape {
        let inputs: Vec<Spelling> = self.inputs.iter().map(|input| input.spelled(inner)).collect();
        MethodShape { name: self.name.clone(), signature: Spelling::signature(&self.generics, &inputs, &self.output) }
    }
}

impl<'head> AsRef<Site<'head>> for SpelledMethod<'head> {
    fn as_ref(&self) -> &Site<'head> {
        &self.site
    }
}
