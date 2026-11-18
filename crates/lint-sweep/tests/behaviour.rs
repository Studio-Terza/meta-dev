use invariants::{invariant, invariants};
use lint_core::{Descend, Facts, Kind, SourceFile, SourceText, Watch, Watching};
use lint_sweep::{Attaches, Attachment, Gathers, Scoped, Sweep};
use syntax::{Block, ExprBinary, Macro};

invariants! {
    enum Invariant {
        FixtureParses => "the nested fixture is valid Rust, so it parses",
    }
}

const NESTED: &str = "fn f() {\n    {\n        {\n        }\n    }\n}\nfn g() {\n    {\n    }\n}\n";

#[derive(Debug, Default)]
struct Recorder {
    depth: usize,
    over_at: Option<usize>,
    heard: Vec<String>,
}

impl Recorder {
    fn passing_over(depth: usize) -> Recorder {
        Recorder { over_at: Some(depth), ..Recorder::default() }
    }
}

impl<'ast> Watch<'ast> for Recorder {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::Block]
    }

    fn on_block(&mut self, _: &'ast Block, _: &Facts<'_, Block>) -> Descend {
        self.depth += 1;
        self.heard.push(format!("on {}", self.depth));
        self.over_at.filter(|at| *at == self.depth).map_or(Descend::Into, |_| Descend::Over)
    }

    fn off_block(&mut self, _: &'ast Block, _: &Facts<'_, Block>) {
        self.heard.push(format!("off {}", self.depth));
        self.depth -= 1;
    }
}

#[test]
fn every_watcher_called_on_a_node_is_called_off_it_and_a_skipped_subtree_is_silent() {
    let file = SourceFile::parse("a.rs", SourceText::new(NESTED.to_owned()));
    let parsed = invariant!(holding file, Invariant::FixtureParses, detailed);
    let mut all = Recorder::default();
    let mut skipping = Recorder::passing_over(2);
    let watchers: Vec<&mut Watching<'_>> = vec![&mut all, &mut skipping];
    Sweep::watching(&parsed, watchers);
    let walked = ["on 1", "on 2", "on 3", "off 3", "off 2", "off 1", "on 1", "on 2", "off 2", "off 1"];
    let skipped_over = ["on 1", "on 2", "off 2", "off 1", "on 1", "on 2", "off 2", "off 1"];
    assert_eq!(all.heard, walked);
    assert_eq!(skipping.heard, skipped_over);
}

const SUMMED: &str = "fn f() {\n    let a = 1 + 2;\n    m!(3 + 4);\n}\n";

#[derive(Debug, Default)]
struct Unfolding {
    sums: usize,
}

#[derive(Debug)]
struct Unattached;

impl Gathers<Unattached> for Unattached {
    fn watching(&mut self) -> impl IntoIterator<Item = &mut Watching<'_>> {
        Vec::new()
    }

    fn gathered(self) -> Unattached {
        self
    }
}

impl Attaches<Unattached> for Unfolding {
    const ATTACHED: &'static [Kind] = &[];

    fn attaching(&mut self) -> Vec<Attachment<Unattached>> {
        Vec::new()
    }

    fn detached(&mut self, _: Unattached) {}
}

impl<'ast> Watch<'ast> for Unfolding {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::Macro, Kind::ExprBinary]
    }

    fn on_macro(&mut self, _: &'ast Macro, _: &Facts<'_, Macro>) -> Descend {
        Descend::Unfold
    }

    fn on_expr_binary(&mut self, _: &'ast ExprBinary, _: &Facts<'_, ExprBinary>) -> Descend {
        self.sums += 1;
        Descend::Into
    }
}

#[test]
fn a_scoped_watcher_that_unfolds_a_macro_hears_what_the_macro_holds() {
    let file = SourceFile::parse("a.rs", SourceText::new(SUMMED.to_owned()));
    let parsed = invariant!(holding file, Invariant::FixtureParses, detailed);
    let mut scoped = Scoped::<Unfolding, Unattached>::new(Unfolding::default());
    Sweep::watching(&parsed, vec![&mut scoped]);
    assert_eq!(scoped.into_parent().sums, 2);
}
