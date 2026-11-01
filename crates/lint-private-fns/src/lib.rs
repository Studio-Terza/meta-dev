mod finding;

use lint_core::Lint;
use std::collections::BTreeMap;

use finding::Finding;
use syn::visit::Visit;
use syn::{ImplItem, ItemImpl, ItemMod, Macro, Visibility};

use lint_core::Macros;
use lint_core::Most;
use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::TypeName;
use lint_core::Violation;
use lint_core::Walk;
use lint_layout::Name;

#[derive(Debug)]
pub struct PrivateFns;

impl Lint for PrivateFns {
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "too many private fns; a type's logic belongs in its interface or in more types",
            files
                .iter()
                .filter(|file| file.is_shipped())
                .flat_map(|file| {
                    Self::private_fns(file.syntax())
                        .into_iter()
                        .filter(|(_, names)| !Self::PER_TYPE.allows(names.len()))
                        .map(move |(type_name, names)| {
                            Violation::new(
                                Site::of_file(file.path()),
                                Finding::TooManyPrivateFns { type_name, names, most: Self::PER_TYPE },
                            )
                        })
                })
                .collect(),
        )
    }
}

impl PrivateFns {
    const PER_TYPE: Most = Most::new(4);

    fn private_fns(file: &syn::File) -> BTreeMap<TypeName, Vec<Name>> {
        let mut counter = PrivateFnCounter::default();
        counter.visit_file(file);
        counter.0
    }
}

#[derive(Default)]
struct PrivateFnCounter(BTreeMap<TypeName, Vec<Name>>);

impl PrivateFnCounter {
    fn private_names(item: &ItemImpl) -> impl Iterator<Item = Name> {
        item.items.iter().filter_map(|member| match member {
            ImplItem::Fn(function) if matches!(function.vis, Visibility::Inherited) => {
                Some(Name::from(&function.sig.ident))
            }
            _ => None,
        })
    }
}

impl<'ast> Visit<'ast> for PrivateFnCounter {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || syn::visit::visit_item_mod(self, module));
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Macros::walk(invocation, |statement| self.visit_stmt(statement));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        Walk::enter(&item.attrs, || match Walk::inherent_impls(item) {
            Walk::Entered => {
                self.0.entry(TypeName::from_type(&item.self_ty)).or_default().extend(Self::private_names(item));
            }
            Walk::Skipped => {}
        });
    }
}
