use lint_file::Filed;
use lint_no_comments::NoComments;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

const COMMENTED: &str = "fn f() {}\n// said\nstruct A; /* also said */\n";

const CASES: [Case; 1] = [Case::new(
    Title::new("each comment after code is flagged on its own line"),
    &[Source::new(Located::new("src/lib.rs"), Text::new(COMMENTED))],
    Flagged::new(&["src/lib.rs:2", "src/lib.rs:3"]),
)];

#[test]
fn comments_are_flagged_on_their_lines() {
    assert_eq!(Sources::mismatches::<Filed<NoComments>>(&CASES), Lines::default());
}
