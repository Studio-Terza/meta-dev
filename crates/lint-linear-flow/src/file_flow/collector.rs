use std::path::Path;
use syntax::{
    Block, ExprBinary, ExprForLoop, ExprIf, ExprLoop, ExprMatch, ExprUnary, ExprWhile, ImplItemFn, Item, ItemFn, Local,
    Macro, Signature, TraitItemFn,
};

use lint_core::{Fold, Folding, SourceFile, Visit};
use lint_sweep::{Descend, Facts, Kind, Sweep};

use lint_core::Violation;

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
}

#[derive(Debug, Default)]
pub struct Flowing<'file> {
    file: Option<&'file SourceFile>,
    frames: Vec<Frame<'file>>,
    closed: Vec<FunctionFlow<'file>>,
}

impl<'file> Collector<'file> {
    pub fn collected(file: &'file SourceFile) -> Vec<FunctionFlow<'file>> {
        Sweep::gathered(file, Folding::from(Collector { file })).closed
    }

    pub fn watching(file: &'file SourceFile) -> Folding<Collector<'file>> {
        Folding::from(Collector { file })
    }

    fn opened(&self, found: Flowing<'file>, signature: &Signature, body: Option<&Block>) -> Visit<Flowing<'file>> {
        let path = AsRef::<Path>::as_ref(self.file.head());
        let opened = body
            .filter(|_| signature.constness.is_none())
            .map(|block| Frame::Function(Opened::new(path, signature.ident.span(), block)));
        Visit::descending(found.pushed(opened.unwrap_or(Frame::Item)))
    }
}

impl<'file> Flowing<'file> {
    fn pushed(self, frame: Frame<'file>) -> Flowing<'file> {
        Flowing { frames: self.frames.into_iter().chain([frame]).collect(), ..self }
    }

    fn shut(self) -> Flowing<'file> {
        let mut frames = self.frames;
        let shut = frames.pop().and_then(Frame::closed);
        Flowing { frames, closed: self.closed.into_iter().chain(shut).collect(), ..self }
    }

    fn topped(self, change: impl FnOnce(Opened<'file>) -> Opened<'file>) -> Flowing<'file> {
        let mut frames = self.frames;
        let top = frames.pop().map(|frame| frame.changed(change));
        Flowing { frames: frames.into_iter().chain(top).collect(), ..self }
    }

    fn inside(self, change: impl FnOnce(Opened<'file>) -> Opened<'file>) -> Visit<Flowing<'file>> {
        Visit::descending(self.topped(|opened| opened.active(change)))
    }
}

impl<'file> Frame<'file> {
    fn closed(self) -> Option<FunctionFlow<'file>> {
        match self {
            Frame::Function(opened) => Some(opened.closed()),
            Frame::Item => None,
        }
    }

    fn changed(self, change: impl FnOnce(Opened<'file>) -> Opened<'file>) -> Frame<'file> {
        match self {
            Frame::Function(opened) => Frame::Function(change(opened)),
            Frame::Item => Frame::Item,
        }
    }
}

impl<'file> Fold for Collector<'file> {
    type Found = Flowing<'file>;

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

    fn finished(&self, found: Flowing<'file>) -> Flowing<'file> {
        Flowing { file: Some(self.file), ..found }
    }

    fn on_item(&self, found: Flowing<'file>, _: &Item, _: &Facts<'_, Item>) -> Visit<Flowing<'file>> {
        Visit::descending(found.pushed(Frame::Item))
    }

    fn off_item(&self, found: Flowing<'file>, _: &Item, _: &Facts<'_, Item>) -> Flowing<'file> {
        let mut frames = found.frames;
        frames.pop();
        Flowing { frames, ..found }
    }

    fn on_item_fn(&self, found: Flowing<'file>, function: &ItemFn, _: &Facts<'_, ItemFn>) -> Visit<Flowing<'file>> {
        self.opened(found, &function.sig, Some(&function.block))
    }

    fn off_item_fn(&self, found: Flowing<'file>, _: &ItemFn, _: &Facts<'_, ItemFn>) -> Flowing<'file> {
        found.shut()
    }

    fn on_impl_item_fn(
        &self,
        found: Flowing<'file>,
        function: &ImplItemFn,
        _: &Facts<'_, ImplItemFn>,
    ) -> Visit<Flowing<'file>> {
        self.opened(found, &function.sig, Some(&function.block))
    }

    fn off_impl_item_fn(&self, found: Flowing<'file>, _: &ImplItemFn, _: &Facts<'_, ImplItemFn>) -> Flowing<'file> {
        found.shut()
    }

    fn on_trait_item_fn(
        &self,
        found: Flowing<'file>,
        function: &TraitItemFn,
        _: &Facts<'_, TraitItemFn>,
    ) -> Visit<Flowing<'file>> {
        self.opened(found, &function.sig, function.default.as_ref())
    }

    fn off_trait_item_fn(&self, found: Flowing<'file>, _: &TraitItemFn, _: &Facts<'_, TraitItemFn>) -> Flowing<'file> {
        found.shut()
    }

    fn on_block(&self, found: Flowing<'file>, block: &Block, _: &Facts<'_, Block>) -> Visit<Flowing<'file>> {
        Visit::descending(found.topped(|opened| opened.entered(block)))
    }

    fn on_macro(&self, found: Flowing<'file>, _: &Macro, _: &Facts<'_, Macro>) -> Visit<Flowing<'file>> {
        Visit::new(found, Descend::Expand)
    }

    fn on_expr_if(&self, found: Flowing<'file>, expr: &ExprIf, _: &Facts<'_, ExprIf>) -> Visit<Flowing<'file>> {
        found.inside(|opened| opened.branched(expr))
    }

    fn on_expr_match(
        &self,
        found: Flowing<'file>,
        expr: &ExprMatch,
        _: &Facts<'_, ExprMatch>,
    ) -> Visit<Flowing<'file>> {
        found.inside(|opened| opened.matched(expr))
    }

    fn on_expr_while(
        &self,
        found: Flowing<'file>,
        expr: &ExprWhile,
        _: &Facts<'_, ExprWhile>,
    ) -> Visit<Flowing<'file>> {
        found.inside(|opened| opened.record(expr.while_token.span, Some(Finding::While)))
    }

    fn on_expr_for_loop(
        &self,
        found: Flowing<'file>,
        expr: &ExprForLoop,
        _: &Facts<'_, ExprForLoop>,
    ) -> Visit<Flowing<'file>> {
        found.inside(|opened| opened.record(expr.for_token.span, Some(Finding::For)))
    }

    fn on_expr_loop(&self, found: Flowing<'file>, expr: &ExprLoop, _: &Facts<'_, ExprLoop>) -> Visit<Flowing<'file>> {
        found.inside(|opened| opened.record(expr.loop_token.span, Some(Finding::Loop)))
    }

    fn on_local(&self, found: Flowing<'file>, local: &Local, _: &Facts<'_, Local>) -> Visit<Flowing<'file>> {
        found.inside(|opened| opened.bound(local))
    }

    fn on_expr_binary(
        &self,
        found: Flowing<'file>,
        expr: &ExprBinary,
        _: &Facts<'_, ExprBinary>,
    ) -> Visit<Flowing<'file>> {
        found.inside(|opened| opened.joined(expr))
    }

    fn on_expr_unary(
        &self,
        found: Flowing<'file>,
        expr: &ExprUnary,
        _: &Facts<'_, ExprUnary>,
    ) -> Visit<Flowing<'file>> {
        found.inside(|opened| opened.negated(expr))
    }
}

impl<'file> From<Flowing<'file>> for Vec<Vec<Violation<'file>>> {
    fn from(flowing: Flowing<'file>) -> Vec<Vec<Violation<'file>>> {
        let tested = flowing
            .file
            .into_iter()
            .flat_map(SourceFile::test_modules)
            .map(|module| FileFlow::collected(module, Collector::collected(module)));
        let own = flowing.file.map(|file| FileFlow::collected(file, flowing.closed));
        let flows: Vec<FileFlow<'file>> = own.into_iter().chain(tested).collect();
        Rule::ALL.iter().map(|rule| FileFlow::violations(&flows, *rule)).collect()
    }
}
