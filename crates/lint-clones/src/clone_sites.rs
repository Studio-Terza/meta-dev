mod finding;

use std::path::Path;

use syn::{ExprMethodCall, ItemMod};

use lint_core::{Gate, Site, Violation, Walk};
use lint_layout::LineNumber;
use lint_sweep::{Descend, Kind, Watch};

use finding::Finding;

#[derive(Debug)]
pub struct CloneSites<'file> {
    path: &'file Path,
    found: Vec<Violation<'file>>,
    kinds: &'static [Kind],
}

impl<'file> CloneSites<'file> {
    const CLONE: &str = "clone";

    pub fn new(path: &'file Path, core: Option<()>) -> CloneSites<'file> {
        let kinds = core.map_or(&[][..], |()| &[Kind::ItemMod, Kind::ExprMethodCall][..]);
        CloneSites { path, found: Vec::new(), kinds }
    }

    pub fn found(self) -> Vec<Violation<'file>> {
        self.found
    }
}

impl<'ast> Watch<'ast> for CloneSites<'_> {
    fn kinds(&self) -> &'static [Kind] {
        self.kinds
    }

    fn on_item_mod(&mut self, module: &'ast ItemMod) -> Descend {
        Gate::BOUNDARY
            .names(module)
            .map_or_else(|| Descend::from(Walk::outside_tests(&module.attrs)), |_| Descend::Over)
    }

    fn on_expr_method_call(&mut self, call: &'ast ExprMethodCall) -> Descend {
        let cloned =
            Some(call).filter(|candidate| candidate.method == Self::CLONE).filter(|clone| clone.args.is_empty()).map(
                |bare_clone| Violation::new(Site::new(self.path, LineNumber::from(bare_clone.method.span())), Finding),
            );
        self.found.extend(cloned);
        Descend::Into
    }
}
