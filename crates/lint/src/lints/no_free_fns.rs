use syn::{ItemFn, Macro};

use crate::finding::{Findings, Found};
use crate::hooks::{Descent, Expanded, Hooks};
use crate::lint::{Lint, Reads};
use crate::source::Source;

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

impl Hooks for NoFreeFns {
    fn item_fn(&mut self, function: &ItemFn) -> Descent {
        let name = &function.sig.ident;
        let free = NoFreeFns::exempt(function).xor(Some(()));
        self.found.extend(free.map(|()| Found::spanned(name.span(), format_args!("fn {name}"), NoFreeFns::FIX)));
        Descent::Into
    }

    fn macro_called(&mut self, _: &Macro) -> Expanded {
        Expanded::Statements
    }
}

impl<'file> Lint<'file> for NoFreeFns {
    const HEADLINE: &'static str = "free functions are not allowed; make them associated functions or methods";
    const READS: Reads = Reads::Every;

    type Fact = Findings;

    fn new(_: &'file Source) -> NoFreeFns {
        NoFreeFns::default()
    }

    fn fact(self, source: &'file Source) -> Findings {
        Findings::found(NoFreeFns::HEADLINE, source.shown(), self.found)
    }
}
