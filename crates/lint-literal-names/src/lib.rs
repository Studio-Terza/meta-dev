mod finding;

use finding::Finding;
use lint_core::Headline;
use lint_core::Joins;
use lint_file::{Reads, SweptLint};
use lint_sweep::{Concludes, Contribute, Contributed, Contributing, Descend, Facts, Kind};
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

    fn watcher(file: &SourceFile) -> Contributing<impl Concludes<'_>> {
        Contributing::new(VagueNames { path: AsRef::<Path>::as_ref(file.head()) })
    }
}

#[derive(Debug)]
struct VagueNames<'file> {
    path: &'file Path,
}

type Noted<'file> = Contributed<(), Vec<Violation<'file>>>;

impl<'file> VagueNames<'file> {
    fn noted(&self, attributes: &[Attribute], ident: &Ident) -> Noted<'file> {
        let descend = Descend::from(Walk::outside_tests(attributes));
        let site = Site::new(self.path, LineNumber::from(ident.span()));
        let name = Name::from(ident);
        let vague = descend.entered().and_then(|()| LiteralNames::vague(&name));
        Contributed::new((), vague.map(|finding| Violation::new(site, finding)).into_iter().collect(), descend)
    }
}

impl<'file> Contribute for VagueNames<'file> {
    type Open = ();
    type Part = Vec<Violation<'file>>;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::Macro, Kind::ItemImpl, Kind::ItemFn, Kind::ImplItemFn]
    }

    fn on_item_mod(
        &self,
        _: &ItemMod,
        facts: &Facts<'_, ItemMod>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Noted<'file> {
        Contributed::descending(Descend::from(facts.gate()))
    }

    fn on_macro(&self, _: &Macro, _: &Facts<'_, Macro>, _around: &lint_sweep::Around<'_, Self::Open>) -> Noted<'file> {
        Contributed::descending(Descend::Expand)
    }

    fn on_item_impl(
        &self,
        _: &ItemImpl,
        facts: &Facts<'_, ItemImpl>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Noted<'file> {
        Contributed::descending(
            Descend::from(facts.gate()).combine(facts.inherent().map_or(Descend::Over, |_| Descend::Into)),
        )
    }

    fn on_item_fn(
        &self,
        function: &ItemFn,
        _: &Facts<'_, ItemFn>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Noted<'file> {
        self.noted(&function.attrs, &function.sig.ident)
    }

    fn on_impl_item_fn(
        &self,
        function: &ImplItemFn,
        _: &Facts<'_, ImplItemFn>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Noted<'file> {
        self.noted(&function.attrs, &function.sig.ident)
    }
}

impl<'file> Concludes<'file> for VagueNames<'file> {
    fn violations(self, (): (), found: Vec<Violation<'file>>) -> Vec<Violation<'file>> {
        found
    }
}
