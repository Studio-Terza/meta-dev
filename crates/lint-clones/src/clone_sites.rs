mod finding;

use std::path::Path;

use syntax::{ExprMethodCall, ItemMod};

use lint_core::{Gate, Site, Violation};
use lint_layout::LineNumber;
use lint_sweep::{Descend, Facts, Fold, Kind, Visit};

use finding::Finding;

#[derive(Debug)]
pub struct CloneSites<'file> {
    path: &'file Path,
    kinds: &'static [Kind],
}

impl<'file> CloneSites<'file> {
    const CLONE: &str = "clone";

    pub fn new(path: &'file Path, core: Option<()>) -> CloneSites<'file> {
        let kinds = core.map_or(&[][..], |()| &[Kind::ItemMod, Kind::ExprMethodCall][..]);
        CloneSites { path, kinds }
    }
}

impl<'file> Fold for CloneSites<'file> {
    type Found = Vec<Violation<'file>>;

    fn kinds(&self) -> &'static [Kind] {
        self.kinds
    }

    fn on_item_mod(&self, found: Self::Found, module: &ItemMod, facts: &Facts<'_, ItemMod>) -> Visit<Self::Found> {
        Visit::new(found, Gate::BOUNDARY.names(module).map_or_else(|| Descend::from(facts.gate()), |_| Descend::Over))
    }

    fn on_expr_method_call(
        &self,
        found: Self::Found,
        call: &ExprMethodCall,
        _: &Facts<'_, ExprMethodCall>,
    ) -> Visit<Self::Found> {
        let cloned =
            Some(call).filter(|candidate| candidate.method == Self::CLONE).filter(|clone| clone.args.is_empty()).map(
                |bare_clone| Violation::new(Site::new(self.path, LineNumber::from(bare_clone.method.span())), Finding),
            );
        Visit::descending(found.into_iter().chain(cloned).collect())
    }
}
