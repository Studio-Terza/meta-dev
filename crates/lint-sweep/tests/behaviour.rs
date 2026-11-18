use invariants::{invariant, invariants};
use lint_core::{Descend, Facts, Kind, SourceFile, SourceText, Watch, Watching};
use lint_sweep::Sweep;
use syntax::Block;

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
