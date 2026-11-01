use proc_macro2::TokenTree;
use syn::visit::{self, Visit};
use syn::{Attribute, ItemFn, Macro, Meta, MetaList};

use lint_core::{LineNumber, Macros, Name, TestMarks, Walk, Whole};

#[derive(Debug, Default)]
pub(crate) struct SkipMarks {
    lines: Vec<LineNumber>,
}

impl SkipMarks {
    const CONDITION: &str = "cfg";
    const CONDITIONAL_ATTRIBUTE: &str = "cfg_attr";
    const SKIPPING: [&str; 2] = ["ignore", "skip"];

    pub(crate) fn lines_in(file: &syn::File) -> Vec<LineNumber> {
        let mut marks = SkipMarks::default();
        marks.visit_file(file);
        marks.lines
    }

    fn names_skip(list: &MetaList) -> bool {
        list.tokens.clone().into_iter().any(|tree| match tree {
            TokenTree::Ident(ident) => Name::from(&ident).matches::<Whole>(&Self::SKIPPING),
            _ => false,
        })
    }

    fn skips(attribute: &Attribute) -> bool {
        match &attribute.meta {
            Meta::List(list) if list.path.is_ident(Self::CONDITIONAL_ATTRIBUTE) => Self::names_skip(list),
            meta => meta
                .path()
                .segments
                .last()
                .is_some_and(|last| Name::from(&last.ident).matches::<Whole>(&Self::SKIPPING)),
        }
    }

    fn record_skips(&mut self, attributes: &[Attribute], marked: Marked) {
        self.lines.extend(
            attributes
                .iter()
                .filter(|attribute| {
                    Self::skips(attribute)
                        || (marked == Marked::Test
                            && Walk::outside_tests(attributes) == Walk::Entered
                            && attribute.path().is_ident(Self::CONDITION))
                })
                .map(|attribute| LineNumber::from(attribute.pound_token.span)),
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Marked {
    Test,
    Other,
}

impl Marked {
    fn among(attributes: &[Attribute], is_law: bool) -> Marked {
        if is_law || TestMarks::mark_test(attributes) { Marked::Test } else { Marked::Other }
    }
}

impl<'ast> Visit<'ast> for SkipMarks {
    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        self.record_skips(&function.attrs, Marked::among(&function.attrs, false));
        visit::visit_item_fn(self, function);
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        let attributes = Macros::attributes(invocation);
        self.record_skips(&attributes, Marked::among(&attributes, TestMarks::is_law(invocation)));
    }
}
