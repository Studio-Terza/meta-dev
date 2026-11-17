use std::path::Path;
use syntax::{
    Block, ExprBinary, ExprForLoop, ExprIf, ExprLoop, ExprMatch, ExprUnary, ExprWhile, ImplItemFn, Item, ItemFn, Local,
    Macro, Signature, TraitItemFn,
};

use lint_core::SourceFile;
use lint_sweep::{Descend, Facts, Kind, Sweep, Watch};

use lint_core::Violation;
use lint_sweep::Sorts;

use super::FileFlow;
use crate::finding::{Finding, Rule};
use crate::function_flow::{FunctionFlow, Opened};

#[derive(Debug)]
enum Frame<'file> {
    Function(Opened<'file>),
    Item,
}

#[derive(Debug)]
pub struct Collector<'file> {
    file: &'file SourceFile,
    frames: Vec<Frame<'file>>,
    closed: Vec<FunctionFlow<'file>>,
}

impl<'file> Collector<'file> {
    pub fn collected(file: &'file SourceFile) -> Vec<FunctionFlow<'file>> {
        let mut collector = Collector { file, frames: Vec::new(), closed: Vec::new() };
        Sweep::watching(file, vec![&mut collector]);
        collector.functions()
    }

    pub fn watching(file: &'file SourceFile) -> Collector<'file> {
        Collector { file, frames: Vec::new(), closed: Vec::new() }
    }

    fn functions(self) -> Vec<FunctionFlow<'file>> {
        self.closed
    }
}

impl<'file> Collector<'file> {
    fn opened(&mut self, signature: &Signature, body: Option<&Block>) -> Descend {
        let path = AsRef::<Path>::as_ref(self.file.head());
        let opened = body
            .filter(|_| signature.constness.is_none())
            .map(|block| Frame::Function(Opened::new(path, signature.ident.span(), block)));
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
    fn closed(self) -> Option<FunctionFlow<'file>> {
        match self {
            Frame::Function(opened) => Some(opened.closed()),
            Frame::Item => None,
        }
    }

    fn opened(&mut self) -> Option<&mut Opened<'file>> {
        match self {
            Frame::Function(opened) => Some(opened),
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

    fn on_item(&mut self, _: &'ast Item, _: &Facts<'_, Item>) -> Descend {
        self.frames.push(Frame::Item);
        Descend::Into
    }

    fn off_item(&mut self, _: &'ast Item, _: &Facts<'_, Item>) {
        self.frames.pop();
    }

    fn on_item_fn(&mut self, function: &'ast ItemFn, _: &Facts<'_, ItemFn>) -> Descend {
        self.opened(&function.sig, Some(&function.block))
    }

    fn off_item_fn(&mut self, _: &'ast ItemFn, _: &Facts<'_, ItemFn>) {
        self.shut();
    }

    fn on_impl_item_fn(&mut self, function: &'ast ImplItemFn, _: &Facts<'_, ImplItemFn>) -> Descend {
        self.opened(&function.sig, Some(&function.block))
    }

    fn off_impl_item_fn(&mut self, _: &'ast ImplItemFn, _: &Facts<'_, ImplItemFn>) {
        self.shut();
    }

    fn on_trait_item_fn(&mut self, function: &'ast TraitItemFn, _: &Facts<'_, TraitItemFn>) -> Descend {
        self.opened(&function.sig, function.default.as_ref())
    }

    fn off_trait_item_fn(&mut self, _: &'ast TraitItemFn, _: &Facts<'_, TraitItemFn>) {
        self.shut();
    }

    fn on_block(&mut self, block: &'ast Block, _: &Facts<'_, Block>) -> Descend {
        self.frames.last_mut().and_then(Frame::opened).into_iter().for_each(|opened| opened.entered(block));
        Descend::Into
    }

    fn on_macro(&mut self, _: &'ast Macro, _: &Facts<'_, Macro>) -> Descend {
        Descend::Expand
    }

    fn on_expr_if(&mut self, expr: &'ast ExprIf, _: &Facts<'_, ExprIf>) -> Descend {
        self.inside().into_iter().for_each(|opened| opened.branched(expr));
        Descend::Into
    }

    fn on_expr_match(&mut self, expr: &'ast ExprMatch, _: &Facts<'_, ExprMatch>) -> Descend {
        self.inside().into_iter().for_each(|opened| opened.matched(expr));
        Descend::Into
    }

    fn on_expr_while(&mut self, expr: &'ast ExprWhile, _: &Facts<'_, ExprWhile>) -> Descend {
        self.inside().into_iter().for_each(|opened| opened.record(expr.while_token.span, Some(Finding::While)));
        Descend::Into
    }

    fn on_expr_for_loop(&mut self, expr: &'ast ExprForLoop, _: &Facts<'_, ExprForLoop>) -> Descend {
        self.inside().into_iter().for_each(|opened| opened.record(expr.for_token.span, Some(Finding::For)));
        Descend::Into
    }

    fn on_expr_loop(&mut self, expr: &'ast ExprLoop, _: &Facts<'_, ExprLoop>) -> Descend {
        self.inside().into_iter().for_each(|opened| opened.record(expr.loop_token.span, Some(Finding::Loop)));
        Descend::Into
    }

    fn on_local(&mut self, local: &'ast Local, _: &Facts<'_, Local>) -> Descend {
        self.inside().into_iter().for_each(|opened| opened.bound(local));
        Descend::Into
    }

    fn on_expr_binary(&mut self, expr: &'ast ExprBinary, _: &Facts<'_, ExprBinary>) -> Descend {
        self.inside().into_iter().for_each(|opened| opened.joined(expr));
        Descend::Into
    }

    fn on_expr_unary(&mut self, expr: &'ast ExprUnary, _: &Facts<'_, ExprUnary>) -> Descend {
        self.inside().into_iter().for_each(|opened| opened.negated(expr));
        Descend::Into
    }
}

impl<'file> Sorts<'file> for Collector<'file> {
    fn sorted(self: Box<Self>) -> Vec<Vec<Violation<'file>>> {
        let file = self.file;
        let tested = file.test_modules().iter().map(|module| FileFlow::collected(module, Collector::collected(module)));
        let flows: Vec<FileFlow<'file>> =
            [FileFlow::collected(file, self.functions())].into_iter().chain(tested).collect();
        Rule::ALL.iter().map(|rule| FileFlow::violations(&flows, *rule)).collect()
    }
}
