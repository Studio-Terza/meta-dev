use proc_macro2::{Ident, TokenStream, TokenTree};
use syn::{Block, ImplItemFn, ItemFn, Macro, Signature, TraitItemFn};

use lint_core::SourceFile;
use lint_sweep::{Descend, Kind, Sweep, Watch};

use lint_core::Violation;
use lint_sweep::Finds;

use super::FileFlow;
use crate::finding::Rule;
use crate::function_flow::FunctionFlow;

#[derive(Debug)]
pub struct Collector<'file> {
    file: &'file SourceFile,
    functions: Vec<FunctionFlow<'file>>,
    rule: Option<Rule>,
}

impl<'file> Collector<'file> {
    const FUNCTION: &'static str = "fn";

    pub fn collected(file: &'file SourceFile) -> Vec<FunctionFlow<'file>> {
        let mut collector = Collector { file, functions: Vec::new(), rule: None };
        Sweep::through_file(file, &mut collector);
        collector.functions
    }

    pub fn ruled(file: &'file SourceFile, rule: Rule) -> Collector<'file> {
        Collector { file, functions: Vec::new(), rule: Some(rule) }
    }

    fn function_keyword(tokens: TokenStream) -> Option<Ident> {
        tokens.into_iter().find_map(Self::keyword_in)
    }

    fn keyword_in(tree: TokenTree) -> Option<Ident> {
        match tree {
            TokenTree::Group(group) => Self::function_keyword(group.stream()),
            TokenTree::Ident(ident) => Some(ident).filter(|found| *found == Self::FUNCTION),
            TokenTree::Punct(_) | TokenTree::Literal(_) => None,
        }
    }

    fn judge(&mut self, signature: &Signature, body: Option<&Block>) -> Descend {
        let file = self.file;
        self.functions.extend(
            body.filter(|_| signature.constness.is_none())
                .map(|block| FunctionFlow::judged(file, signature.ident.span(), block)),
        );
        Descend::Into
    }
}

impl<'ast> Watch<'ast> for Collector<'_> {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::Macro, Kind::ItemFn, Kind::ImplItemFn, Kind::TraitItemFn]
    }

    fn on_macro(&mut self, invocation: &'ast Macro) -> Descend {
        Self::function_keyword(invocation.tokens.clone()).map_or(Descend::Over, |_| Descend::Expand)
    }

    fn on_item_fn(&mut self, function: &'ast ItemFn) -> Descend {
        self.judge(&function.sig, Some(&function.block))
    }

    fn on_impl_item_fn(&mut self, function: &'ast ImplItemFn) -> Descend {
        self.judge(&function.sig, Some(&function.block))
    }

    fn on_trait_item_fn(&mut self, function: &'ast TraitItemFn) -> Descend {
        self.judge(&function.sig, function.default.as_ref())
    }
}

impl<'file> Finds<'file> for Collector<'file> {
    fn found(self: Box<Self>) -> Vec<Violation<'file>> {
        let file = self.file;
        let tested = file.test_modules().iter().map(|module| FileFlow::collected(module, Collector::collected(module)));
        let flows: Vec<FileFlow<'file>> =
            [FileFlow::collected(file, self.functions)].into_iter().chain(tested).collect();
        self.rule.map_or_default(|rule| FileFlow::violations(&flows, rule))
    }
}
