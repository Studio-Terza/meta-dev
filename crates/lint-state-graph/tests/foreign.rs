use lint_fixtures::{Case, Flagged, Lines, Located, Queried, Source, Sources, Text, Title};
use lint_state_graph::ForeignMatches;

const PORTS: Source = Source::new(
    Located::new("crates/ports/src/lib.rs"),
    Text::new("mod watch;\n\npub use watch::{Stall, Watched};\n"),
);
const WATCH: Source = Source::new(
    Located::new("crates/ports/src/watch.rs"),
    Text::new("pub enum Watched {\n    Stalled(u8),\n    Finished,\n}\n"),
);
const USER: Source = Source::new(
    Located::new("crates/user/src/lib.rs"),
    Text::new(
        "use ports::{Other, Watched};\n\nimpl Seer {\n    fn seen(&self, watched: Watched) -> Result<u8, F> {\n        match watched {\n            Watched::Stalled(at) => Ok(Step::Ended(at)),\n            Watched::Finished => self.read(1),\n        }\n    }\n}\n",
    ),
);

const CASES: [Case; 1] = [Case::new(
    Title::new("a re-exported sum type of another crate"),
    &[PORTS, WATCH, USER],
    Flagged::new(&["crates/user/src/lib.rs:6", "crates/user/src/lib.rs:7"]),
)];

#[test]
fn foreign_matches_are_flagged() {
    assert_eq!(Sources::mismatches::<Queried<ForeignMatches>>(&CASES), Lines::default());
}
