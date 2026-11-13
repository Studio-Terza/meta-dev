mod finding;

use lint_core::Headline;
use lint_file::{Reads, SweptLint};
use lint_sweep::{Descend, Finds, Kind, Watch};
use std::collections::BTreeMap;

use finding::Finding;
use syn::{ImplItem, ImplItemFn, ItemImpl, ItemMod, Macro, Visibility};

use lint_core::Bound;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::TypeName;
use lint_core::Violation;
use lint_core::Walk;
use lint_layout::{Length, Name};

#[derive(Debug)]
pub struct PrivateFns;

impl SweptLint for PrivateFns {
    const HEADLINE: Headline =
        Headline::new("too many private fns; a type's logic belongs in its interface or in more types");
    const READS: Reads = Reads::Shipped;

    fn watcher(file: &SourceFile) -> Box<dyn Finds<'_> + '_> {
        Box::new(PrivateFnCounter { file, found: BTreeMap::new() })
    }
}

impl PrivateFns {
    const PER_TYPE: Bound = Bound::Most(4);

    fn violations(file: &SourceFile, found: BTreeMap<TypeName, Vec<Name>>) -> Vec<Violation<'_>> {
        found
            .into_iter()
            .filter(|(_, names)| Self::PER_TYPE.allows(Length::new(names.len())).is_none())
            .map(|(type_name, names)| {
                Violation::new(
                    Site::of_file(file.head().path()),
                    Finding::TooManyPrivateFns { type_name, names, most: Self::PER_TYPE },
                )
            })
            .collect()
    }
}

#[derive(Debug)]
struct PrivateFnCounter<'file> {
    file: &'file SourceFile,
    found: BTreeMap<TypeName, Vec<Name>>,
}

impl PrivateFnCounter<'_> {
    fn private_names(item: &ItemImpl) -> impl Iterator<Item = Name> {
        item.items
            .iter()
            .filter_map(Self::function)
            .filter(|function| matches!(function.vis, Visibility::Inherited))
            .map(|function| Name::from(&function.sig.ident))
    }

    fn function(member: &ImplItem) -> Option<&ImplItemFn> {
        match member {
            ImplItem::Fn(function) => Some(function),
            _ => None,
        }
    }

    fn inherent(&mut self, item: &ItemImpl) -> Descend {
        let counted =
            Descend::from(Walk::outside_tests(&item.attrs)).then(|| Descend::from(Walk::inherent_impls(item)));
        let private = counted.entered().map(|_| Self::private_names(item).collect::<Vec<Name>>());
        self.found.entry(TypeName::from_type(&item.self_ty)).or_default().extend(private.into_iter().flatten());
        Descend::Over
    }
}

impl<'ast> Watch<'ast> for PrivateFnCounter<'_> {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::Macro, Kind::ItemImpl]
    }

    fn on_item_mod(&mut self, module: &'ast ItemMod) -> Descend {
        Descend::from(Walk::outside_tests(&module.attrs))
    }

    fn on_macro(&mut self, _: &'ast Macro) -> Descend {
        Descend::Expand
    }

    fn on_item_impl(&mut self, item: &'ast ItemImpl) -> Descend {
        self.inherent(item)
    }
}

impl<'file> Finds<'file> for PrivateFnCounter<'file> {
    fn found(self: Box<Self>) -> Vec<Violation<'file>> {
        PrivateFns::violations(self.file, self.found)
    }
}
