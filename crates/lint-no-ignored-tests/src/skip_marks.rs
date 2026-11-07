mod gate;

use proc_macro2::{Ident, TokenTree};
use syn::visit::{self, Visit};
use syn::{Attribute, ItemFn, ItemMod, Macro, MetaList};

use gate::Gate;
use lint_core::Macros;
use lint_layout::Name;
use lint_layout::TestMarks;
use lint_layout::Whole;
use lint_layout::{LineNumber, Word, words};

#[derive(Debug, Default)]
pub(crate) struct SkipMarks {
    lines: Vec<LineNumber>,
    gates: Vec<LineNumber>,
}

impl SkipMarks {
    const CONDITIONAL_ATTRIBUTE: &str = "cfg_attr";
    const SKIPPING: [Word<'static>; 2] = words!["ignore", "skip"];

    pub(crate) fn lines_in(file: &syn::File) -> Vec<LineNumber> {
        let mut marks = Self::default();
        marks.gates.extend(Gate::lines_in(&file.attrs));
        marks.visit_file(file);
        marks.lines.sort();
        marks.lines.dedup();
        marks.lines
    }

    fn skip_named(list: &MetaList) -> Option<Word<'static>> {
        list.tokens
            .clone()
            .into_iter()
            .filter_map(Self::ident)
            .find_map(|ident| Name::from(&ident).matches::<Whole>(&Self::SKIPPING))
    }

    fn ident(tree: TokenTree) -> Option<Ident> {
        match tree {
            TokenTree::Ident(ident) => Some(ident),
            _ => None,
        }
    }

    fn skip(attribute: &Attribute) -> Option<Word<'static>> {
        attribute.meta.require_list().ok().filter(|list| list.path.is_ident(Self::CONDITIONAL_ATTRIBUTE)).map_or_else(
            || {
                attribute
                    .meta
                    .path()
                    .segments
                    .last()
                    .and_then(|last| Name::from(&last.ident).matches::<Whole>(&Self::SKIPPING))
            },
            Self::skip_named,
        )
    }

    fn record_skips(&mut self, attributes: &[Attribute], marked: Marked) {
        self.lines.extend(
            attributes
                .iter()
                .filter_map(|attribute| Self::skip(attribute).map(|_| LineNumber::from(attribute.pound_token.span))),
        );
        self.lines.extend(
            Gate::lines_in(attributes).into_iter().chain(self.gates.iter().copied()).filter(|_| marked == Marked::Test),
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Marked {
    Test,
    Other,
}

impl Marked {
    fn among(attributes: &[Attribute], law: Option<&Macro>) -> Self {
        law.map(|_| Self::Test).or_else(|| TestMarks::mark_test(attributes).map(|_| Self::Test)).unwrap_or(Self::Other)
    }
}

impl<'ast> Visit<'ast> for SkipMarks {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let before = self.gates.len();
        self.gates.extend(Gate::lines_in(&module.attrs));
        visit::visit_item_mod(self, module);
        self.gates.truncate(before);
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        self.record_skips(&function.attrs, Marked::among(&function.attrs, None));
        visit::visit_item_fn(self, function);
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        let attributes = Macros::attributes(invocation);
        self.record_skips(&attributes, Marked::among(&attributes, TestMarks::law(invocation)));
    }
}
