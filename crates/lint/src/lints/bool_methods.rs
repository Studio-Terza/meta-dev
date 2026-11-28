use syn::visit::Visit;
use syn::{ItemImpl, Macro, ReturnType, Signature, TypePath};

use crate::finding::{Found, Rule};
use crate::gate::Gate;
use crate::hooks::{Descent, Expanded, Hooks};
use crate::reached::Reached;
use crate::source::Source;
use crate::style::{Reads, Style};
use crate::terms::Fix;

#[derive(Debug, Default)]
pub(crate) struct BoolMethods {
    found: Vec<Found>,
}

impl BoolMethods {
    const FIX: Fix =
        Fix("return an enum to match on, an Option or Result carrying the value, or a proof type such as NonEmpty");

    fn flag(path: &TypePath) -> Option<()> {
        let flag = path.path.segments.last().filter(|segment| segment.ident == "bool" && segment.arguments.is_none());
        flag.and(path.qself.is_none().then_some(()))
    }
}

impl Hooks for BoolMethods {
    fn macro_called(&mut self, _: &Macro) -> Expanded {
        Expanded::Statements
    }

    fn item_impl(&mut self, item: &ItemImpl) -> Descent {
        Descent::from(Gate::inherent(item).entered())
    }

    fn signature(&mut self, signature: &Signature) -> Descent {
        let flagged = match &signature.output {
            ReturnType::Type(_, returned) => {
                Reached::within(|reached| reached.visit_type(returned)).paths().find_map(BoolMethods::flag)
            }
            ReturnType::Default => None,
        };
        let name = &signature.ident;
        self.found.extend(
            flagged.map(|()| Found::spanned(name.span(), format_args!("fn {name} returns bool"), BoolMethods::FIX)),
        );
        Descent::Over
    }
}

impl<'file> Style<'file> for BoolMethods {
    const RULE: Rule = Rule::BoolMethods;
    const READS: Reads = Reads::Every;

    fn new(_: &'file Source) -> BoolMethods {
        BoolMethods::default()
    }

    fn found(self) -> Vec<Found> {
        self.found
    }
}
