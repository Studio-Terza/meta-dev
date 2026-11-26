use std::path::Path;
mod finding;

use algebra::{Monoid, Semigroup};
use lint_core::Headline;
use lint_file::{Reads, SweptLint};
use lint_sweep::{Concludes, Contribute, Contributed, Contributing, Descend, Facts, Finds, Kind};
use std::collections::BTreeMap;

use finding::Finding;
use syntax::{ImplItem, ImplItemFn, ItemImpl, ItemMod, Macro, Visibility};

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

    fn watcher(file: &SourceFile) -> impl Finds<'_> {
        Contributing::new(PrivateFnCounter { file })
    }
}

impl PrivateFns {
    const PER_TYPE: Bound = Bound::Most(4);

    fn violations(file: &SourceFile, found: PrivateNames) -> Vec<Violation<'_>> {
        found
            .0
            .into_iter()
            .filter(|(_, names)| Self::PER_TYPE.allows(Length::new(names.len())).is_none())
            .map(|(type_name, names)| {
                Violation::new(
                    Site::of_file(AsRef::<Path>::as_ref(file.head())),
                    Finding::TooManyPrivateFns { type_name, names, most: Self::PER_TYPE },
                )
            })
            .collect()
    }
}

#[derive(Debug)]
struct PrivateFnCounter<'file> {
    file: &'file SourceFile,
}

#[derive(Debug, Default)]
struct PrivateNames(BTreeMap<TypeName, Vec<Name>>);

impl Semigroup for PrivateNames {
    fn combine(mut self, other: PrivateNames) -> PrivateNames {
        other.0.into_iter().for_each(|(type_name, names)| self.0.entry(type_name).or_default().extend(names));
        self
    }
}

impl Monoid for PrivateNames {}

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
}

impl Contribute for PrivateFnCounter<'_> {
    type Open = ();
    type Part = PrivateNames;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::Macro, Kind::ItemImpl]
    }

    fn on_item_mod(
        &self,
        _: &ItemMod,
        facts: &Facts<'_, ItemMod>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), PrivateNames> {
        Contributed::descending(Descend::from(facts.gate()))
    }

    fn on_macro(
        &self,
        _: &Macro,
        _: &Facts<'_, Macro>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), PrivateNames> {
        Contributed::descending(Descend::Expand)
    }

    fn on_item_impl(
        &self,
        item: &ItemImpl,
        _: &Facts<'_, ItemImpl>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), PrivateNames> {
        let counted =
            Descend::from(Walk::outside_tests(&item.attrs)).combine(Descend::from(Walk::inherent_impls(item)));
        let private: Vec<Name> = counted
            .entered()
            .map(|()| Self::private_names(item).collect::<Vec<Name>>())
            .into_iter()
            .flatten()
            .collect();
        let names = PrivateNames(BTreeMap::from([(TypeName::from_type(&item.self_ty), private)]));
        Contributed::new((), names, Descend::Over)
    }
}

impl<'file> Concludes<'file> for PrivateFnCounter<'file> {
    fn violations(self, (): (), found: PrivateNames) -> Vec<Violation<'file>> {
        PrivateFns::violations(self.file, found)
    }
}
