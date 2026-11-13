mod finding;

use lint_core::{Headline, Lint};
use lint_file::{FileLint, Reads};
use std::collections::BTreeMap;

use finding::Finding;
use syn::visit::Visit;
use syn::{ImplItem, ImplItemFn, ItemImpl, ItemMod, Macro, Visibility};

use lint_core::Bound;
use lint_core::Macros;
use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::TypeName;
use lint_core::Violation;
use lint_core::Walk;
use lint_layout::{Length, Name};

#[derive(Debug)]
pub struct PrivateFns;

impl Lint for PrivateFns {
    fn report(files: &[SourceFile]) -> Report<'_> {
        <Self as FileLint>::reported(files)
    }
}

impl FileLint for PrivateFns {
    const HEADLINE: Headline =
        Headline::new("too many private fns; a type's logic belongs in its interface or in more types");
    const READS: Reads = Reads::Shipped;

    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        Self::private_fns(file)
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

impl PrivateFns {
    const PER_TYPE: Bound = Bound::Most(4);

    fn private_fns(file: &SourceFile) -> BTreeMap<TypeName, Vec<Name>> {
        let mut counter = PrivateFnCounter { file, found: BTreeMap::new() };
        counter.visit_file(file.syntax());
        counter.found
    }
}

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

    fn inherent(&mut self, item: &ItemImpl) {
        match Walk::inherent_impls(item) {
            Walk::Entered => {
                self.found.entry(TypeName::from_type(&item.self_ty)).or_default().extend(Self::private_names(item));
            }
            Walk::Skipped => {}
        }
    }
}

impl<'ast> Visit<'ast> for PrivateFnCounter<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || syn::visit::visit_item_mod(self, module));
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Macros::walk(self.file, invocation, |statement| self.visit_stmt(statement));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        Walk::enter(&item.attrs, || self.inherent(item));
    }
}
