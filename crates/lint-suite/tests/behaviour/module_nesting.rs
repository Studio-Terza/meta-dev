use lint_module_nesting::ModuleNesting;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

const OUTER: Source = Source::new(Located::new("crates/a/src/outer/mod.rs"), Text::new("pub mod inner;\n"));
const INNER: Source = Source::new(Located::new("crates/a/src/outer/inner.rs"), Text::new("pub struct Thing;\n"));
const REACHING: Source =
    Source::new(Located::new("crates/a/src/user.rs"), Text::new("use crate::outer::inner::Thing;\n"));
const THROUGH: Source = Source::new(Located::new("crates/a/src/user.rs"), Text::new("use crate::outer::Thing;\n"));
const OTHER_THROUGH: Source =
    Source::new(Located::new("crates/a/src/other.rs"), Text::new("use crate::outer::Thing;\n"));

const BARE_ROOT: Source =
    Source::new(Located::new("crates/a/src/lib.rs"), Text::new("mod outer;\nuse outer::inner::Thing;\n"));

const OUTER_FILE: Source = Source::new(Located::new("crates/a/src/outer.rs"), Text::new("pub mod inner;\n"));
const FAMILY: Source = Source::new(
    Located::new("crates/a/src/outer/mod.rs"),
    Text::new("pub mod inner;\npub mod first;\npub mod second;\n"),
);
const FIRST: Source =
    Source::new(Located::new("crates/a/src/outer/first.rs"), Text::new("use crate::outer::inner::Thing;\n"));
const SECOND: Source =
    Source::new(Located::new("crates/a/src/outer/second.rs"), Text::new("use crate::outer::inner::Thing;\n"));

const CASES: [Case; 4] = [
    Case::new(Title::new("through the parent"), &[OUTER, INNER, THROUGH, OTHER_THROUGH], Flagged::new(&[])),
    Case::new(Title::new("a parent without mod.rs"), &[OUTER_FILE, INNER, REACHING], Flagged::new(&[])),
    Case::new(Title::new("users inside the boundary"), &[FAMILY, INNER, FIRST, SECOND], Flagged::new(&[])),
    Case::new(
        Title::new("a bare path past mod.rs"),
        &[BARE_ROOT, OUTER, INNER],
        Flagged::new(&["crates/a/src/lib.rs:1"]),
    ),
];

#[test]
fn a_use_path_reaching_past_a_directory_module_is_flagged() {
    assert_eq!(Sources::mismatches::<ModuleNesting>(&CASES), Lines::default());
}
