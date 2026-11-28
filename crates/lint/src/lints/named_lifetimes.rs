use syn::{LifetimeParam, Macro};

use crate::finding::{Found, Rule};
use crate::hooks::{Descent, Expanded, Hooks};
use crate::source::Source;
use crate::style::{Reads, Style};
use crate::terms::Fix;

#[derive(Debug, Default)]
pub(crate) struct NamedLifetimes {
    found: Vec<Found>,
}

impl NamedLifetimes {
    const SHORTEST: usize = 2;
    const FIX: Fix = Fix("name the lifetime with a word that says what it borrows from");
}

impl Hooks for NamedLifetimes {
    fn lifetime_param(&mut self, parameter: &LifetimeParam) -> Descent {
        let name = &parameter.lifetime.ident;
        let short = name.to_string().chars().count() < NamedLifetimes::SHORTEST;
        self.found.extend(short.then(|| Found::spanned(name.span(), format_args!("'{name}"), NamedLifetimes::FIX)));
        Descent::Into
    }

    fn macro_called(&mut self, _: &Macro) -> Expanded {
        Expanded::Statements
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
