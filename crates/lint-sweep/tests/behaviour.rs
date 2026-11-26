use std::fmt;

use invariants::{invariant, invariants};
use lint_core::Joins;
use lint_core::{SourceFile, SourceText, Tallied};
use lint_layout::Name;
use lint_sweep::{
    Around, Contribute, Contributed, Contributing, Contribution, Descend, Facts, Joined, Joint, Kind, Scope, Unjoined,
};
use syntax::{Block, ExprBinary, Ident, Macro};

invariants! {
    enum Invariant {
        FixtureParses => "the nested fixture is valid Rust, so it parses",
    }
}

const NESTED: &str = "fn f() {\n    {\n        {\n        }\n    }\n}\nfn g() {\n    {\n    }\n}\n";

#[derive(Debug, Default)]
struct Depth(Option<usize>);

impl Joins for Depth {
    fn combine(self, other: Depth) -> Depth {
        Depth(self.0.or(other.0))
    }
}

#[derive(Debug)]
struct Recorder {
    over_at: Option<usize>,
}

impl Recorder {
    const SCOPES: [Scope; 1] = [Scope::new(Kind::Block)];
}

impl Contribute for Recorder {
    type Open = Depth;
    type Part = Vec<String>;

    fn kinds(&self) -> &'static [Kind] {
        &[]
    }

    fn scopes(&self) -> &'static [Scope] {
        &Recorder::SCOPES
    }

    fn on_block(&self, _: &Block, _: &Facts<'_, Block>, around: &Around<'_, Depth>) -> Contributed<Depth, Vec<String>> {
        let depth = around.nearest().count() + 1;
        let descend = self.over_at.filter(|at| *at == depth).map_or(Descend::Into, |_| Descend::Over);
        Contributed::new(Depth(Some(depth)), vec![format!("on {depth}")], descend)
    }

    fn off_block(&self, _: &Block, _: &Facts<'_, Block>, inner: Contribution<Depth, Vec<String>>) -> Vec<String> {
        inner.concluded(|depth, heard| heard.into_iter().chain(depth.0.map(|closed| format!("off {closed}"))).collect())
    }
}

#[test]
fn every_scope_entered_is_closed_and_a_skipped_subtree_is_silent() {
    let file = SourceFile::parse("a.rs", SourceText::new(NESTED.to_owned()));
    let parsed = invariant!(holding file, Invariant::FixtureParses, detailed);
    let all = Contributing::new(Recorder { over_at: None }).concluded(&parsed, |_, _, heard| heard);
    let skipping = Contributing::new(Recorder { over_at: Some(2) }).concluded(&parsed, |_, _, heard| heard);
    let walked = ["on 1", "on 2", "on 3", "off 3", "off 2", "off 1", "on 1", "on 2", "off 2", "off 1"];
    let skipped_over = ["on 1", "on 2", "off 2", "off 1", "on 1", "on 2", "off 2", "off 1"];
    assert_eq!(all, walked);
    assert_eq!(skipping, skipped_over);
}

const SUMMED: &str = "fn f() {\n    let a = 1 + 2;\n    m!(3 + 4);\n}\n";

#[derive(Debug)]
struct Unfolding;

impl Contribute for Unfolding {
    type Open = ();
    type Part = Vec<()>;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::Macro, Kind::ExprBinary]
    }

    fn on_macro(&self, _: &Macro, _: &Facts<'_, Macro>, _: &Around<'_, ()>) -> Contributed<(), Vec<()>> {
        Contributed::descending(Descend::Unfold)
    }

    fn on_expr_binary(
        &self,
        _: &ExprBinary,
        _: &Facts<'_, ExprBinary>,
        _: &Around<'_, ()>,
    ) -> Contributed<(), Vec<()>> {
        Contributed::parted(vec![()])
    }
}

const JOINED: &str = "fn f() {\n    {\n        let a = 1 + 2;\n        m!({ 3 + 4 });\n        {\n        }\n    }\n}\nfn g() {\n    {\n    }\n}\n";

#[derive(Debug)]
struct Positions;

impl Positions {
    const SCOPES: [Scope; 2] = [Scope::new(Kind::Ident), Scope::new(Kind::Macro)];
}

impl Contribute for Positions {
    type Open = ();
    type Part = Vec<String>;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::Block, Kind::ExprBinary, Kind::Macro, Kind::Ident]
    }

    fn scopes(&self) -> &'static [Scope] {
        &Positions::SCOPES
    }

    fn on_ident(&self, _: &Ident, name: &Name, around: &Around<'_, ()>) -> Contributed<(), Vec<String>> {
        Contributed::parted(vec![format!("{name} at {:?}", around.position())])
    }

    fn off_ident(&self, found: &Ident, inner: Contribution<(), Vec<String>>) -> Vec<String> {
        inner.concluded(|(), heard| heard.into_iter().chain([format!("left {found}")]).collect())
    }

    fn on_block(&self, _: &Block, _: &Facts<'_, Block>, around: &Around<'_, ()>) -> Contributed<(), Vec<String>> {
        Contributed::parted(vec![format!("block at {:?}", around.position())])
    }

    fn on_expr_binary(
        &self,
        _: &ExprBinary,
        _: &Facts<'_, ExprBinary>,
        around: &Around<'_, ()>,
    ) -> Contributed<(), Vec<String>> {
        Contributed::parted(vec![format!("sum at {:?}", around.position())])
    }

    fn on_macro(&self, _: &Macro, _: &Facts<'_, Macro>, around: &Around<'_, ()>) -> Contributed<(), Vec<String>> {
        Contributed::new((), vec![format!("macro at {:?}", around.position())], Descend::Expand)
    }

    fn off_macro(&self, _: &Macro, _: &Facts<'_, Macro>, inner: Contribution<(), Vec<String>>) -> Vec<String> {
        inner.concluded(|(), heard| heard.into_iter().chain(["left the macro".to_owned()]).collect())
    }
}

#[derive(Debug)]
struct Heard;

impl Heard {
    fn told<Contributor: Contribute<Open: fmt::Debug, Part: fmt::Debug>>(tallied: Tallied<Contributor>) -> String {
        tallied.concluded(|_, open, part, visits| format!("{open:?} {part:?} after {visits} visits"))
    }

    fn alone<Contributor: Contribute<Open: fmt::Debug, Part: fmt::Debug>>(
        file: &SourceFile,
        contributor: Contributor,
    ) -> String {
        Heard::told(Contributing::new(contributor).tallied(file))
    }
}

#[test]
fn one_joint_walk_tells_each_member_what_its_own_walk_would() {
    let file = SourceFile::parse("a.rs", SourceText::new(JOINED.to_owned()));
    let parsed = invariant!(holding file, Invariant::FixtureParses, detailed);
    let missing: Option<Contributing<Positions>> = None;
    let joint = Joined::new(
        Some(Contributing::new(Recorder { over_at: None })),
        Joined::new(
            Some(Contributing::new(Recorder { over_at: Some(2) })),
            Joined::new(
                missing,
                Joined::new(
                    Some(Contributing::new(Unfolding)),
                    Joined::new(Some(Contributing::new(Positions)), Unjoined),
                ),
            ),
        ),
    );
    let told = joint.swept(&parsed).concluded(|all, second| {
        second.concluded(|skipping, third| {
            third.concluded(|absent, fourth| {
                fourth.concluded(|unfolding, fifth| {
                    fifth.concluded(|positions, Unjoined| {
                        [all.map(Heard::told), skipping.map(Heard::told), absent.map(Heard::told)]
                            .into_iter()
                            .chain([unfolding.map(Heard::told), positions.map(Heard::told)])
                            .collect::<Vec<Option<String>>>()
                    })
                })
            })
        })
    });
    let alone = vec![
        Some(Heard::alone(&parsed, Recorder { over_at: None })),
        Some(Heard::alone(&parsed, Recorder { over_at: Some(2) })),
        None,
        Some(Heard::alone(&parsed, Unfolding)),
        Some(Heard::alone(&parsed, Positions)),
    ];
    assert_eq!(told, alone);
}

#[test]
fn a_contributor_that_unfolds_a_macro_hears_what_the_macro_holds() {
    let file = SourceFile::parse("a.rs", SourceText::new(SUMMED.to_owned()));
    let parsed = invariant!(holding file, Invariant::FixtureParses, detailed);
    let sums = Contributing::new(Unfolding).concluded(&parsed, |_, (), sums| sums);
    assert_eq!(sums.len(), 2);
}
