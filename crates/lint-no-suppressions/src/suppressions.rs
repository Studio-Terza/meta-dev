use proc_macro2::TokenTree;
use syn::visit::{self, Visit};
use syn::{Attribute, Meta};

use lint_layout::LineNumber;

#[derive(Debug, Default)]
pub(crate) struct Suppressions {
    lines: Vec<LineNumber>,
}

impl Suppressions {
    const SWITCHES: [&'static str; 2] = ["allow", "expect"];
    const CONDITIONAL: &'static str = "cfg_attr";

    pub(crate) fn lines_in(file: &syn::File) -> Vec<LineNumber> {
        let mut found = Suppressions::default();
        found.visit_file(file);
        found.lines.sort();
        found.lines.dedup();
        found.lines
    }

    fn switch(ident: &proc_macro2::Ident) -> bool {
        Self::SWITCHES.iter().any(|name| ident == name)
    }

    fn switches_off(attribute: &Attribute) -> bool {
        match &attribute.meta {
            Meta::List(list) if list.path.is_ident(Self::CONDITIONAL) => list
                .tokens
                .clone()
                .into_iter()
                .any(|tree| matches!(tree, TokenTree::Ident(ident) if Self::switch(&ident))),
            meta => meta.path().get_ident().is_some_and(Self::switch),
        }
    }
}

impl<'ast> Visit<'ast> for Suppressions {
    fn visit_attribute(&mut self, attribute: &'ast Attribute) {
        self.lines.extend(Self::switches_off(attribute).then(|| LineNumber::from(attribute.pound_token.span)));
        visit::visit_attribute(self, attribute);
    }
}
