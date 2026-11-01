use syn::visit::Visit;
use syn::{ImplItem, ImplItemFn, ItemImpl, ItemMod, ReturnType, Signature, Type, Visibility};

use lint_core::{Site, SourceFile, Violation, Walk};
use lint_layout::{LineNumber, Name, Prefix};

use super::Finding;

pub struct Predicates<'file> {
    file: &'file SourceFile,
    found: Vec<Violation>,
}

impl Predicates<'_> {
    const PREFIXES: [&'static str; 2] = ["is_", "has_"];

    pub fn of_file(file: &SourceFile) -> Vec<Violation> {
        let mut scan = Predicates { file, found: Vec::new() };
        scan.visit_file(file.syntax());
        scan.found
    }

    fn in_block(&self, block: &ItemImpl) -> Vec<Violation> {
        match (Walk::inherent_impls(block), Walk::outside_tests(&block.attrs)) {
            (Walk::Entered, Walk::Entered) => {
                block.items.iter().filter_map(Self::predicate).map(|method| self.violation(method)).collect()
            }
            _ => Vec::new(),
        }
    }

    fn predicate(item: &ImplItem) -> Option<&ImplItemFn> {
        match item {
            ImplItem::Fn(method)
                if matches!(method.vis, Visibility::Public(_))
                    && Name::from(&method.sig.ident).matches::<Prefix>(&Self::PREFIXES)
                    && Self::returns_bool(&method.sig) =>
            {
                Some(method)
            }
            _ => None,
        }
    }

    fn returns_bool(signature: &Signature) -> bool {
        match &signature.output {
            ReturnType::Type(_, returned) => {
                matches!(returned.as_ref(), Type::Path(path) if path.path.is_ident("bool"))
            }
            ReturnType::Default => false,
        }
    }

    fn violation(&self, method: &ImplItemFn) -> Violation {
        let name = &method.sig.ident;
        Violation::new(Site::new(self.file.path(), LineNumber::from(name.span())), Finding::from(name))
    }
}

impl<'ast> Visit<'ast> for Predicates<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || syn::visit::visit_item_mod(self, module));
    }

    fn visit_item_impl(&mut self, block: &'ast ItemImpl) {
        self.found.extend(self.in_block(block));
    }
}
