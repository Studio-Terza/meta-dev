use lint_core::{Descend, Facts, Fold, Folding, Kind, SourceFile, SourceText, Visit, Watching};
use lint_sweep::Sweep;
use syntax::Block;

const NESTED: &str = "fn f() {\n    {\n        {\n        }\n    }\n}\nfn g() {\n    {\n    }\n}\n";

#[derive(Debug, Default)]
struct Recorder {
    over_at: Option<usize>,
}

#[derive(Debug, Default)]
struct Heard {
    depth: usize,
    heard: Vec<String>,
}

impl Fold for Recorder {
    type Found = Heard;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::Block]
    }

    fn on_block(&self, found: Heard, _: &Block, _: &Facts<'_, Block>) -> Visit<Heard> {
        let depth = found.depth + 1;
        let descend = self.over_at.filter(|at| *at == depth).map_or(Descend::Into, |_| Descend::Over);
        let heard = found.heard.into_iter().chain([format!("on {depth}")]).collect();
        Visit::new(Heard { depth, heard }, descend)
    }

    fn off_block(&self, found: Heard, _: &Block, _: &Facts<'_, Block>) -> Heard {
        let heard = found.heard.into_iter().chain([format!("off {}", found.depth)]).collect();
        Heard { depth: found.depth - 1, heard }
    }
}

#[test]
fn every_watcher_called_on_a_node_is_called_off_it_and_a_skipped_subtree_is_silent() {
    let file = SourceFile::parse("a.rs", SourceText::new(NESTED.to_owned()));
    let parsed = file.unwrap_or_else(|failure| unreachable!("the source parses: {failure}"));
    let mut all = Folding::from(Recorder::default());
    let mut skipping = Folding::from(Recorder { over_at: Some(2) });
    let watchers: Vec<&mut Watching<'_>> = vec![&mut all, &mut skipping];
    Sweep::watching(&parsed, watchers);
    let walked = ["on 1", "on 2", "on 3", "off 3", "off 2", "off 1", "on 1", "on 2", "off 2", "off 1"];
    let skipped_over = ["on 1", "on 2", "off 2", "off 1", "on 1", "on 2", "off 2", "off 1"];
    assert_eq!(all.concluded::<Heard>().heard, walked);
    assert_eq!(skipping.concluded::<Heard>().heard, skipped_over);
}
