use lint_core::Joins;
use invariants::{invariant, invariants};
use lint_core::{SourceFile, SourceText};
use lint_sweep::{Around, Contribute, Contributed, Contributing, Contribution, Descend, Facts, Kind, Scope};
use syntax::{Block, ExprBinary, Macro};

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

#[test]
fn a_contributor_that_unfolds_a_macro_hears_what_the_macro_holds() {
    let file = SourceFile::parse("a.rs", SourceText::new(SUMMED.to_owned()));
    let parsed = invariant!(holding file, Invariant::FixtureParses, detailed);
    let sums = Contributing::new(Unfolding).concluded(&parsed, |_, (), sums| sums);
    assert_eq!(sums.len(), 2);
}
