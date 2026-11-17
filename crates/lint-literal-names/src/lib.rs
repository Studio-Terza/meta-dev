mod finding;

use finding::Finding;
use lint_core::Extended;
use lint_core::Headline;
use lint_file::{Reads, SweptLint};
use lint_sweep::{Descend, Facts, Finds, Fold, Folding, Kind, Visit};
use std::path::Path;

use syntax::{Attribute, Ident, ImplItemFn, ItemFn, ItemImpl, ItemMod, Macro};

use lint_core::Bound;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::Walk;
use lint_layout::LineNumber;
use lint_layout::{Name, Whole, Word, words};

#[derive(Debug)]
pub struct LiteralNames;

impl LiteralNames {
    const SHORTEST: Bound = Bound::Least(3);
    const VAGUE: [Word<'static>; 24] = words![
        "with", "from", "into", "for", "get", "set", "make", "check", "build", "create", "init", "process", "handle",
        "helper", "util", "data", "value", "item", "thing", "execute", "perform", "update", "compute", "manage",
    ];

    fn vague(name: &Name) -> Option<Finding<'_>> {
        Self::SHORTEST
            .allows(name.length())
            .is_none()
            .then_some(Finding::VagueName { name })
            .or_else(|| name.matches::<Whole>(&Self::VAGUE).map(|_| Finding::VagueName { name }))
    }
}

impl SweptLint for LiteralNames {
    const HEADLINE: Headline = Headline::new("function names say what the function does");
    const READS: Reads = Reads::Shipped;

    fn watcher(file: &SourceFile) -> Box<dyn Finds<'_> + '_> {
        Box::new(Folding::from(VagueNames { path: AsRef::<Path>::as_ref(file.head()) }))
    }
}

#[derive(Debug)]
struct VagueNames<'file> {
    path: &'file Path,
}

impl<'file> VagueNames<'file> {
    fn noted(
        &self,
        found: Vec<Violation<'file>>,
        attributes: &[Attribute],
        ident: &Ident,
    ) -> Visit<Vec<Violation<'file>>> {
        let descend = Descend::from(Walk::outside_tests(attributes));
        let site = Site::new(self.path, LineNumber::from(ident.span()));
        let name = Name::from(ident);
        let vague = descend.entered().and_then(|_| LiteralNames::vague(&name));
        let noted = found.extended(vague.map(|finding| Violation::new(site, finding)));
        Visit::new(noted, descend)
    }
}

impl<'file> Fold for VagueNames<'file> {
    type Found = Vec<Violation<'file>>;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::Macro, Kind::ItemImpl, Kind::ItemFn, Kind::ImplItemFn]
    }

    fn on_item_mod(&self, found: Self::Found, _: &ItemMod, facts: &Facts<'_, ItemMod>) -> Visit<Self::Found> {
        Visit::new(found, Descend::from(facts.gate()))
    }

    fn on_macro(&self, found: Self::Found, _: &Macro, _: &Facts<'_, Macro>) -> Visit<Self::Found> {
        Visit::new(found, Descend::Expand)
    }

    fn on_item_impl(&self, found: Self::Found, _: &ItemImpl, facts: &Facts<'_, ItemImpl>) -> Visit<Self::Found> {
        Visit::new(
            found,
            Descend::from(facts.gate()).then(|| facts.inherent().map_or(Descend::Over, |_| Descend::Into)),
        )
    }

    fn on_item_fn(&self, found: Self::Found, function: &ItemFn, _: &Facts<'_, ItemFn>) -> Visit<Self::Found> {
        self.noted(found, &function.attrs, &function.sig.ident)
    }

    fn on_impl_item_fn(
        &self,
        found: Self::Found,
        function: &ImplItemFn,
        _: &Facts<'_, ImplItemFn>,
    ) -> Visit<Self::Found> {
        self.noted(found, &function.attrs, &function.sig.ident)
    }
}
