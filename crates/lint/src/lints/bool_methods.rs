use syn::visit::{self, Visit};
use syn::{ItemImpl, Macro, ReturnType, Signature, TypePath};

use crate::finding::{Found, Rule};
use crate::gate::Gate;
use crate::source::Source;
use crate::style::{Reads, Style};
use crate::walk::{Descent, Expanded, Hooks};

#[derive(Debug, Default)]
pub(crate) struct BoolMethods {
    found: Vec<Found>,
}

#[derive(Debug, Default)]
struct Flags(Option<()>);

impl BoolMethods {
    const FIX: &'static str =
        "return an enum to match on, an Option or Result carrying the value, or a proof type such as NonEmpty";
}

impl<'ast> Visit<'ast> for Flags {
    fn visit_type_path(&mut self, path: &'ast TypePath) {
        let flag = path.path.segments.last().filter(|segment| segment.ident == "bool" && segment.arguments.is_none());
        self.0 = self.0.or(flag.and(path.qself.is_none().then_some(())));
        visit::visit_type_path(self, path);
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
                let mut flags = Flags::default();
                flags.visit_type(returned);
                flags.0
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
