mod boundary;

use proc_macro2::{Ident, TokenStream, TokenTree};
use syn::{
    Block, ExprBinary, ExprForLoop, ExprIf, ExprLoop, ExprMatch, ExprUnary, ExprWhile, ImplItemFn, Item, ItemFn, Local,
    Macro, Signature, TraitItemFn,
};

use lint_core::SourceFile;
use lint_sweep::{Descend, Kind, Sweep, Watch};

use lint_core::Violation;
use lint_sweep::Finds;

use super::FileFlow;
use crate::finding::{Finding, Rule};
use crate::function_flow::{FunctionFlow, Opened};
use boundary::Order;

#[derive(Debug)]
enum Frame<'file> {
    Function { order: Order, opened: Opened<'file> },
    Item,
}

#[derive(Clone, Copy, Debug)]
struct Keyword;

impl Keyword {
    const FUNCTION: &'static str = "fn";

    fn in_tokens(tokens: TokenStream) -> Option<Ident> {
        tokens.into_iter().find_map(Self::in_tree)
    }

    fn in_tree(tree: TokenTree) -> Option<Ident> {
        match tree {
            TokenTree::Group(group) => Self::in_tokens(group.stream()),
            TokenTree::Ident(ident) => Some(ident).filter(|found| *found == Self::FUNCTION),
            TokenTree::Punct(_) | TokenTree::Literal(_) => None,
        }
    }
}

#[derive(Debug)]
struct Closed<'file> {
    order: Order,
    flow: FunctionFlow<'file>,
}

#[derive(Debug)]
pub struct Collector<'file> {
    file: &'file SourceFile,
    rule: Option<Rule>,
    next: Order,
    frames: Vec<Frame<'file>>,
    closed: Vec<Closed<'file>>,
}

impl<'file> Collector<'file> {
    pub fn collected(file: &'file SourceFile) -> Vec<FunctionFlow<'file>> {
        let mut collector =
            Collector { file, rule: None, next: Order::default(), frames: Vec::new(), closed: Vec::new() };
        Sweep::through_file(file, &mut collector);
        collector.functions()
    }

    pub fn ruled(file: &'file SourceFile, rule: Rule) -> Collector<'file> {
        Collector { file, rule: Some(rule), next: Order::default(), frames: Vec::new(), closed: Vec::new() }
    }

    fn functions(self) -> Vec<FunctionFlow<'file>> {
        let mut closed = self.closed;
        closed.sort_by_key(|function| function.order);
        closed.into_iter().map(|function| function.flow).collect()
    }
}

impl<'file> Collector<'file> {
    fn opened(&mut self, signature: &Signature, body: Option<&Block>) -> Descend {
        let path = self.file.head().path();
        let order = self.next;
        let opened = body
            .filter(|_| signature.constness.is_none())
            .map(|block| Frame::Function { order, opened: Opened::new(path, signature.ident.span(), block) });
        self.next = opened.as_ref().map_or(self.next, |_| self.next.next());
        self.frames.push(opened.unwrap_or(Frame::Item));
        Descend::Into
    }

    fn shut(&mut self) {
        let shut = self.frames.pop().and_then(Frame::closed);
        self.closed.extend(shut);
    }

    fn inside(&mut self) -> Option<&mut Opened<'file>> {
        self.frames.last_mut().and_then(Frame::opened).and_then(Opened::active)
    }
}

impl<'file> Frame<'file> {
    fn closed(self) -> Option<Closed<'file>> {
        match self {
            Frame::Function { order, opened } => Some(Closed { order, flow: opened.closed() }),
            Frame::Item => None,
        }
    }

    fn opened(&mut self) -> Option<&mut Opened<'file>> {
        match self {
            Frame::Function { opened, .. } => Some(opened),
            Frame::Item => None,
        }
    }
}

impl<'ast> Watch<'ast> for Collector<'_> {
    fn kinds(&self) -> &'static [Kind] {
        &[
            Kind::Item,
            Kind::ItemFn,
            Kind::ImplItemFn,
            Kind::TraitItemFn,
            Kind::Block,
            Kind::Macro,
            Kind::ExprIf,
            Kind::ExprMatch,
            Kind::ExprWhile,
            Kind::ExprForLoop,
            Kind::ExprLoop,
            Kind::Local,
            Kind::ExprBinary,
            Kind::ExprUnary,
        ]
    }

    fn on_item(&mut self, _: &'ast Item) -> Descend {
        self.frames.push(Frame::Item);
        Descend::Into
    }

    fn off_item(&mut self, _: &'ast Item) {
        self.frames.pop();
    }

    fn on_item_fn(&mut self, function: &'ast ItemFn) -> Descend {
        self.opened(&function.sig, Some(&function.block))
    }

    fn off_item_fn(&mut self, _: &'ast ItemFn) {
        self.shut();
    }

    fn on_impl_item_fn(&mut self, function: &'ast ImplItemFn) -> Descend {
        self.opened(&function.sig, Some(&function.block))
    }

    fn off_impl_item_fn(&mut self, _: &'ast ImplItemFn) {
        self.shut();
    }

    fn on_trait_item_fn(&mut self, function: &'ast TraitItemFn) -> Descend {
        self.opened(&function.sig, function.default.as_ref())
    }

    fn off_trait_item_fn(&mut self, _: &'ast TraitItemFn) {
        self.shut();
    }

    fn on_block(&mut self, block: &'ast Block) -> Descend {
        self.frames.last_mut().and_then(Frame::opened).into_iter().for_each(|opened| opened.entered(block));
        Descend::Into
    }

    fn on_macro(&mut self, invocation: &'ast Macro) -> Descend {
        let keyword = Keyword::in_tokens(invocation.tokens.clone()).map(|_| Descend::Expand);
        let body = self.inside().map(|_| Descend::Expand);
        body.or(keyword).unwrap_or(Descend::Over)
    }

    fn on_expr_if(&mut self, expr: &'ast ExprIf) -> Descend {
        self.inside().into_iter().for_each(|opened| opened.branched(expr));
        Descend::Into
    }

    fn on_expr_match(&mut self, expr: &'ast ExprMatch) -> Descend {
        self.inside().into_iter().for_each(|opened| opened.matched(expr));
        Descend::Into
    }

    fn on_expr_while(&mut self, expr: &'ast ExprWhile) -> Descend {
        self.inside().into_iter().for_each(|opened| opened.record(expr.while_token.span, Some(Finding::While)));
        Descend::Into
    }

    fn on_expr_for_loop(&mut self, expr: &'ast ExprForLoop) -> Descend {
        self.inside().into_iter().for_each(|opened| opened.record(expr.for_token.span, Some(Finding::For)));
        Descend::Into
    }

    fn on_expr_loop(&mut self, expr: &'ast ExprLoop) -> Descend {
        self.inside().into_iter().for_each(|opened| opened.record(expr.loop_token.span, Some(Finding::Loop)));
        Descend::Into
    }

    fn on_local(&mut self, local: &'ast Local) -> Descend {
        self.inside().into_iter().for_each(|opened| opened.bound(local));
        Descend::Into
    }

    fn on_expr_binary(&mut self, expr: &'ast ExprBinary) -> Descend {
        self.inside().into_iter().for_each(|opened| opened.joined(expr));
        Descend::Into
    }

    fn on_expr_unary(&mut self, expr: &'ast ExprUnary) -> Descend {
        self.inside().into_iter().for_each(|opened| opened.negated(expr));
        Descend::Into
    }
}

impl<'file> Finds<'file> for Collector<'file> {
    fn found(self: Box<Self>) -> Vec<Violation<'file>> {
        let file = self.file;
        let rule = self.rule;
        let tested = file.test_modules().iter().map(|module| FileFlow::collected(module, Collector::collected(module)));
        let flows: Vec<FileFlow<'file>> =
            [FileFlow::collected(file, self.functions())].into_iter().chain(tested).collect();
        rule.map_or_default(|ruled| FileFlow::violations(&flows, ruled))
    }
}
