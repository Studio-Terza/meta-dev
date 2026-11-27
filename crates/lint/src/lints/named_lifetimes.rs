use syn::visit::{self, Visit};
use syn::{LifetimeParam, Macro};

use crate::expansion::Expansion;
use crate::finding::{Fix, Found, Rule};
use crate::source::Source;
use crate::style::{Reads, Style};

#[derive(Debug, Default)]
pub(crate) struct NamedLifetimes {
    found: Vec<Found>,
}

impl NamedLifetimes {
    const SHORTEST: usize = 2;
    const FIX: Fix = Fix("name the lifetime with a word that says what it borrows from");
}

impl<'ast> Visit<'ast> for NamedLifetimes {
    fn visit_lifetime_param(&mut self, parameter: &'ast LifetimeParam) {
        let name = &parameter.lifetime.ident;
        let short = name.to_string().chars().count() < NamedLifetimes::SHORTEST;
        self.found.extend(short.then(|| Found::spanned(name.span(), format_args!("'{name}"), NamedLifetimes::FIX)));
        visit::visit_lifetime_param(self, parameter);
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Expansion::of(invocation).walk(self);
    }
}

impl<'file> Style<'file> for NamedLifetimes {
    const RULE: Rule = Rule::NamedLifetimes;
    const READS: Reads = Reads::Every;

    fn new(_: &'file Source) -> NamedLifetimes {
        NamedLifetimes::default()
    }

    fn found(self) -> Vec<Found> {
        self.found
    }
}
