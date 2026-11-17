use std::path::Path;
mod finding;

use lint_core::Headline;
use lint_file::{Reads, SweptLint};
use lint_sweep::{Descend, Facts, Finds, Fold, Folding, Kind, Visit};
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

    fn watcher(file: &SourceFile) -> Box<dyn Finds<'_> + '_> {
        Box::new(Folding::from(PrivateFnCounter { file }))
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
struct Counted<'file> {
    file: Option<&'file SourceFile>,
    by_type: BTreeMap<TypeName, Vec<Name>>,
}

impl<'file> From<Counted<'file>> for Vec<Violation<'file>> {
    fn from(counted: Counted<'file>) -> Vec<Violation<'file>> {
        counted.file.map_or_default(|file| PrivateFns::violations(file, counted.by_type))
    }
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

    fn inherent<'file>(found: Counted<'file>, item: &ItemImpl) -> Visit<Counted<'file>> {
        let counted =
            Descend::from(Walk::outside_tests(&item.attrs)).then(|| Descend::from(Walk::inherent_impls(item)));
        let private = counted.entered().map(|_| Self::private_names(item).collect::<Vec<Name>>());
        let Counted { file, mut by_type } = found;
        by_type.entry(TypeName::from_type(&item.self_ty)).or_default().extend(private.into_iter().flatten());
        Visit::new(Counted { file, by_type }, Descend::Over)
    }
}

impl<'file> Fold for PrivateFnCounter<'file> {
    type Found = Counted<'file>;

    fn finished(&self, found: Counted<'file>) -> Counted<'file> {
        Counted { file: Some(self.file), ..found }
    }

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::Macro, Kind::ItemImpl]
    }

    fn on_item_mod(&self, found: Self::Found, _: &ItemMod, facts: &Facts<'_, ItemMod>) -> Visit<Self::Found> {
        Visit::new(found, Descend::from(facts.gate()))
    }

    fn on_macro(&self, found: Self::Found, _: &Macro, _: &Facts<'_, Macro>) -> Visit<Self::Found> {
        Visit::new(found, Descend::Expand)
    }

    fn on_item_impl(&self, found: Self::Found, item: &ItemImpl, _: &Facts<'_, ItemImpl>) -> Visit<Self::Found> {
        Self::inherent(found, item)
    }
}
