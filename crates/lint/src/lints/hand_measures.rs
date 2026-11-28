use syn::ItemImpl;

use crate::finding::{Found, Rule};
use crate::hooks::{Descent, Hooks};
use crate::source::Source;
use crate::style::{Reads, Style};

#[derive(Debug)]
pub(crate) struct HandMeasures {
    found: Vec<Found>,
    home: Option<()>,
}

impl HandMeasures {
    const TRAIT: &'static str = "Measure";
    const HOME: &'static str = "pipeline";
}

impl Hooks for HandMeasures {
    fn item_impl(&mut self, item: &ItemImpl) -> Descent {
        let measured = item.trait_.as_ref().and_then(|(_, path, _)| path.segments.last());
        let hand = measured.filter(|last| last.ident == HandMeasures::TRAIT).filter(|_| self.home.is_none());
        self.found.extend(hand.map(|last| {
            Found::spanned(
                last.ident.span(),
                "hand-written impl Measure",
                "derive Measure; only pipeline joins std types by hand",
            )
        }));
        Descent::Into
    }
}

impl<'file> Style<'file> for HandMeasures {
    const RULE: Rule = Rule::HandMeasures;
    const READS: Reads = Reads::Every;

    fn new(source: &'file Source) -> HandMeasures {
        let home = (source.package().name() == HandMeasures::HOME).then_some(());
        HandMeasures { found: Vec::new(), home }
    }

    fn found(self) -> Vec<Found> {
        self.found
    }
}
