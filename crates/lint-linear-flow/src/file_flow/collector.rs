use std::path::Path;

use syntax::{
    Block, ExprBinary, ExprForLoop, ExprIf, ExprLoop, ExprMatch, ExprUnary, ExprWhile, ImplItemFn, Item, ItemFn, Local,
    Macro, Signature, TraitItemFn,
};

use algebra::{Monoid, Semigroup};
use lint_core::SourceFile;
use lint_sweep::{Contribute, Contributed, Contributing, Contribution, Descend, Facts, Kind, Ranked, Ranks, Scope};
use strum::VariantArray;

use super::FileFlow;
use crate::finding::{Finding, Rule};
use crate::function_flow::{Candidate, FunctionFlow};

#[derive(Debug)]
pub struct Collector<'file> {
    file: &'file SourceFile,
}

#[derive(Debug, Default)]
pub struct Flows<'file> {
    closed: Vec<FunctionFlow<'file>>,
    pending: Vec<Candidate<'file>>,
}

impl Semigroup for Flows<'_> {
    fn combine(self, other: Self) -> Self {
        Flows { closed: self.closed.combine(other.closed), pending: self.pending.combine(other.pending) }
    }
}

impl Monoid for Flows<'_> {}

impl<'file> Flows<'file> {
    fn settled(self) -> Flows<'file> {
        Flows { closed: self.closed, pending: Vec::new() }
    }
}

impl<'file> Collector<'file> {
    pub fn collected(file: &'file SourceFile) -> Vec<FunctionFlow<'file>> {
        Contributing::new(Collector { file }).concluded(file, |_, (), flows| flows.closed)
    }

    pub fn watching(file: &'file SourceFile) -> Contributing<Collector<'file>> {
        Contributing::new(Collector { file })
    }

    fn path(&self) -> &'file Path {
        AsRef::<Path>::as_ref(self.file.head())
    }

    fn closed(
        &self,
        signature: &Signature,
        body: Option<&Block>,
        inner: Contribution<(), Flows<'file>>,
    ) -> Flows<'file> {
        inner.concluded(|(), flows| {
            let Flows { closed, pending } = flows;
            let tracked = body.filter(|_| signature.constness.is_none());
            let flow = tracked.map(|block| FunctionFlow::closed(self.path(), signature.ident.span(), block, pending));
            Flows { closed: closed.into_iter().chain(flow).collect(), pending: Vec::new() }
        })
    }

    fn candidate(found: Option<Candidate<'file>>) -> Contributed<(), Flows<'file>> {
        Contributed::parted(Flows { closed: Vec::new(), pending: found.into_iter().collect() })
    }
}

impl<'file> Contribute for Collector<'file> {
    type Open = ();
    type Part = Flows<'file>;

    fn kinds(&self) -> &'static [Kind] {
        &[
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

    fn scopes(&self) -> &'static [Scope] {
        const {
            &[
                Scope::new(Kind::Item),
                Scope::new(Kind::ItemFn),
                Scope::new(Kind::ImplItemFn),
                Scope::new(Kind::TraitItemFn),
            ]
        }
    }

    fn off_item(&self, _: &Item, _: &Facts<'_, Item>, inner: Contribution<(), Flows<'file>>) -> Flows<'file> {
        inner.concluded(|(), flows| flows.settled())
    }

    fn off_item_fn(
        &self,
        function: &ItemFn,
        _: &Facts<'_, ItemFn>,
        inner: Contribution<(), Flows<'file>>,
    ) -> Flows<'file> {
        self.closed(&function.sig, Some(&function.block), inner)
    }

    fn off_impl_item_fn(
        &self,
        function: &ImplItemFn,
        _: &Facts<'_, ImplItemFn>,
        inner: Contribution<(), Flows<'file>>,
    ) -> Flows<'file> {
        self.closed(&function.sig, Some(&function.block), inner)
    }

    fn off_trait_item_fn(
        &self,
        function: &TraitItemFn,
        _: &Facts<'_, TraitItemFn>,
        inner: Contribution<(), Flows<'file>>,
    ) -> Flows<'file> {
        self.closed(&function.sig, function.default.as_ref(), inner)
    }

    fn on_macro(
        &self,
        _: &Macro,
        _: &Facts<'_, Macro>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Flows<'file>> {
        Contributed::descending(Descend::Expand)
    }

    fn on_expr_if(
        &self,
        expr: &ExprIf,
        _: &Facts<'_, ExprIf>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Flows<'file>> {
        Self::candidate(Candidate::branched(self.path(), expr))
    }

    fn on_expr_match(
        &self,
        expr: &ExprMatch,
        _: &Facts<'_, ExprMatch>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Flows<'file>> {
        Self::candidate(Candidate::matched(self.path(), expr))
    }

    fn on_expr_while(
        &self,
        expr: &ExprWhile,
        _: &Facts<'_, ExprWhile>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Flows<'file>> {
        Self::candidate(Candidate::looped(self.path(), expr.while_token.span, Finding::While))
    }

    fn on_expr_for_loop(
        &self,
        expr: &ExprForLoop,
        _: &Facts<'_, ExprForLoop>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Flows<'file>> {
        Self::candidate(Candidate::looped(self.path(), expr.for_token.span, Finding::For))
    }

    fn on_expr_loop(
        &self,
        expr: &ExprLoop,
        _: &Facts<'_, ExprLoop>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Flows<'file>> {
        Self::candidate(Candidate::looped(self.path(), expr.loop_token.span, Finding::Loop))
    }

    fn on_local(
        &self,
        local: &Local,
        _: &Facts<'_, Local>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Flows<'file>> {
        Self::candidate(Candidate::bound(self.path(), local))
    }

    fn on_expr_binary(
        &self,
        expr: &ExprBinary,
        _: &Facts<'_, ExprBinary>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Flows<'file>> {
        Self::candidate(Candidate::joined(self.path(), expr))
    }

    fn on_expr_unary(
        &self,
        expr: &ExprUnary,
        _: &Facts<'_, ExprUnary>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Flows<'file>> {
        Self::candidate(Candidate::negated(self.path(), expr))
    }
}

impl<'file> Ranks<'file> for Collector<'file> {
    fn ranked(self, (): (), gathered: Flows<'file>) -> Ranked<'file> {
        let file = self.file;
        let tested = file.test_modules().iter().map(|module| FileFlow::collected(module, Collector::collected(module)));
        let flows: Vec<FileFlow<'file>> =
            [FileFlow::collected(file, gathered.closed)].into_iter().chain(tested).collect();
        Ranked::from(Rule::VARIANTS.iter().map(|rule| FileFlow::violations(&flows, *rule)).collect::<Vec<_>>())
    }
}
