use proc_macro2::Span;
use syn::{Item, ItemFn};

use lint_core::SourceFile;
use lint_sweep::{Descend, Kind, Sweep, Watch};

use crate::ThinMain;

#[derive(Debug, Default)]
pub(crate) struct BodyItems {
    found: Vec<Span>,
}

impl BodyItems {
    fn declared(item: &Item) -> Option<Span> {
        match item {
            Item::Use(_) => None,
            _ => Some(ThinMain::keyword(item)),
        }
    }

    pub(crate) fn declared_in(file: &SourceFile, main: &ItemFn) -> Vec<Span> {
        let mut items = BodyItems::default();
        Sweep::through_block(file, &main.block, &mut items);
        items.found
    }
}

impl<'ast> Watch<'ast> for BodyItems {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::Item]
    }

    fn on_item(&mut self, item: &'ast Item) -> Descend {
        self.found.extend(Self::declared(item));
        Descend::Over
    }
}
