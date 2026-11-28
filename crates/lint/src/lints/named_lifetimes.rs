use syn::{LifetimeParam, Macro};

use crate::finding::{Findings, Found};
use crate::hooks::{Descent, Expanded, Hooks};
use crate::lint::{Lint, Reads};
use crate::source::Source;

#[derive(Debug, Default)]
pub(crate) struct NamedLifetimes {
    found: Vec<Found>,
}

impl NamedLifetimes {
    const SHORTEST: usize = 2;
    const FIX: &'static str = "name the lifetime with a word that says what it borrows from";
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

impl<'file> Lint<'file> for NamedLifetimes {
    const HEADLINE: &'static str = "lifetimes are named with a word, never a letter";
    const READS: Reads = Reads::Every;

    type Fact = Findings;

    fn new(_: &'file Source) -> NamedLifetimes {
        NamedLifetimes::default()
    }

    fn fact(self, source: &'file Source) -> Findings {
        Findings::found(NamedLifetimes::HEADLINE, source.shown(), self.found)
    }
}
