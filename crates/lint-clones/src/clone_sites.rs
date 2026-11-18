mod finding;

use std::path::Path;

use syntax::{ExprMethodCall, ItemMod};

use lint_core::{Gate, Site, Violation};
use lint_layout::LineNumber;
use lint_sweep::{Contribute, Contributed, Descend, Facts, Kind};

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

impl<'file> Contribute for CloneSites<'file> {
    type Open = ();
    type Part = Vec<Violation<'file>>;

    fn kinds(&self) -> &'static [Kind] {
        self.kinds
    }

    fn on_item_mod(
        &self,
        module: &ItemMod,
        facts: &Facts<'_, ItemMod>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Vec<Violation<'file>>> {
        Contributed::descending(Gate::BOUNDARY.names(module).chosen(Descend::Over, Descend::from(facts.gate())))
    }

    fn on_expr_method_call(
        &self,
        call: &ExprMethodCall,
        _: &Facts<'_, ExprMethodCall>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Vec<Violation<'file>>> {
        let cloned =
            Some(call).filter(|candidate| candidate.method == Self::CLONE).filter(|clone| clone.args.is_empty()).map(
                |bare_clone| Violation::new(Site::new(self.path, LineNumber::from(bare_clone.method.span())), Finding),
            );
        Contributed::parted(cloned.into_iter().collect())
    }
}
