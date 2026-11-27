use syn::visit::{self, Visit};
use syn::{ItemFn, Macro};

use crate::expansion::Expansion;
use crate::finding::{Found, Rule};
use crate::source::Source;
use crate::style::{Reads, Style};

#[derive(Debug, Default)]
pub(crate) struct NoFreeFns {
    found: Vec<Found>,
}

impl NoFreeFns {
    const REQUIRED_FREE: [&'static str; 4] = ["test", "proc_macro", "proc_macro_derive", "proc_macro_attribute"];
    const FIX: &'static str = "make it an associated function or a method";

    fn exempt(function: &ItemFn) -> Option<()> {
        let main = (function.sig.ident == "main").then_some(());
        main.or_else(|| {
            function
                .attrs
                .iter()
                .filter_map(|attribute| attribute.path().segments.last())
                .find(|segment| NoFreeFns::REQUIRED_FREE.iter().any(|required| segment.ident == required))
                .map(|_| ())
        })
    }
}

impl<'ast> Visit<'ast> for NoFreeFns {
    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        let name = &function.sig.ident;
        let free = NoFreeFns::exempt(function).xor(Some(()));
        self.found.extend(free.map(|()| Found::spanned(name.span(), format_args!("fn {name}"), NoFreeFns::FIX)));
        visit::visit_item_fn(self, function);
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Expansion::of(invocation).walk(self);
    }
}

impl<'file> Style<'file> for NoFreeFns {
    const RULE: Rule = Rule::NoFreeFns;
    const READS: Reads = Reads::Every;

    fn new(_: &'file Source) -> NoFreeFns {
        NoFreeFns::default()
    }

    fn found(self) -> Vec<Found> {
        self.found
    }
}
