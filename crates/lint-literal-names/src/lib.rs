mod finding;

use finding::Finding;
use lint_core::Headline;
use lint_file::{Reads, SweptLint};
use lint_sweep::{Descend, Finds, Kind, Watch};
use std::path::Path;

use syn::{Attribute, Ident, ImplItemFn, ItemFn, ItemImpl, ItemMod, Macro};

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
        Box::new(VagueNames { path: file.head().path(), found: Vec::new() })
    }
}

struct VagueNames<'file> {
    path: &'file Path,
    found: Vec<Violation<'file>>,
}

impl VagueNames<'_> {
    fn noted(&mut self, attributes: &[Attribute], ident: &Ident) -> Descend {
        let descend = Descend::from(Walk::outside_tests(attributes));
        let site = Site::new(self.path, LineNumber::from(ident.span()));
        let name = Name::from(ident);
        let vague = descend.entered().and_then(|_| LiteralNames::vague(&name));
        self.found.extend(vague.map(|finding| Violation::new(site, finding)));
        descend
    }
}

impl<'ast> Watch<'ast> for VagueNames<'_> {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::Macro, Kind::ItemImpl, Kind::ItemFn, Kind::ImplItemFn]
    }

    fn on_item_mod(&mut self, module: &'ast ItemMod) -> Descend {
        Descend::from(Walk::outside_tests(&module.attrs))
    }

    fn on_macro(&mut self, _: &'ast Macro) -> Descend {
        Descend::Expand
    }

    fn on_item_impl(&mut self, item: &'ast ItemImpl) -> Descend {
        Descend::from(Walk::outside_tests(&item.attrs)).then(|| Descend::from(Walk::inherent_impls(item)))
    }

    fn on_item_fn(&mut self, function: &'ast ItemFn) -> Descend {
        self.noted(&function.attrs, &function.sig.ident)
    }

    fn on_impl_item_fn(&mut self, function: &'ast ImplItemFn) -> Descend {
        self.noted(&function.attrs, &function.sig.ident)
    }
}

impl<'file> Finds<'file> for VagueNames<'file> {
    fn found(self: Box<Self>) -> Vec<Violation<'file>> {
        self.found
    }
}
