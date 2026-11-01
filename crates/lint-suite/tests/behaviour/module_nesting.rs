use lint_module_nesting::ModuleNesting;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

const ROOT: Source = Source::new(Located::new("crates/a/src/lib.rs"), Text::new("mod parent;\nmod helper;\n"));
const PARENT: Source = Source::new(Located::new("crates/a/src/parent.rs"), Text::new("use crate::helper::Tool;\n"));
const SECOND_PARENT: Source =
    Source::new(Located::new("crates/a/src/other.rs"), Text::new("use crate::helper::Tool;\n"));
const HELPER: Source = Source::new(Located::new("crates/a/src/helper.rs"), Text::new("pub struct Tool;\n"));
const OUTER: Source = Source::new(Located::new("crates/a/src/outer/mod.rs"), Text::new("pub mod inner;\n"));
const INNER: Source = Source::new(Located::new("crates/a/src/outer/inner.rs"), Text::new("pub struct Thing;\n"));
const REACHING: Source =
    Source::new(Located::new("crates/a/src/user.rs"), Text::new("use crate::outer::inner::Thing;\n"));
const THROUGH: Source = Source::new(Located::new("crates/a/src/user.rs"), Text::new("use crate::outer::Thing;\n"));
const OTHER_THROUGH: Source =
    Source::new(Located::new("crates/a/src/other.rs"), Text::new("use crate::outer::Thing;\n"));

const HIDDEN_WIDE: Source = Source::new(
    Located::new("crates/a/src/wide.rs"),
    Text::new("pub(crate) use crate::x::{A, B, C, D, E, F, G, H, I};\n"),
);

const OUTER_FILE: Source = Source::new(Located::new("crates/a/src/outer.rs"), Text::new("pub mod inner;\n"));
const FAMILY: Source = Source::new(
    Located::new("crates/a/src/outer/mod.rs"),
    Text::new("pub mod inner;\npub mod first;\npub mod second;\n"),
);
const FIRST: Source =
    Source::new(Located::new("crates/a/src/outer/first.rs"), Text::new("use crate::outer::inner::Thing;\n"));
const SECOND: Source =
    Source::new(Located::new("crates/a/src/outer/second.rs"), Text::new("use crate::outer::inner::Thing;\n"));

const CASES: [Case; 5] = [
    Case::new(Title::new("used by two siblings"), &[ROOT, PARENT, SECOND_PARENT, HELPER], Flagged::new(&[])),
    Case::new(Title::new("through the parent"), &[OUTER, INNER, THROUGH, OTHER_THROUGH], Flagged::new(&[])),
    Case::new(Title::new("crate-only re-exports"), &[HIDDEN_WIDE], Flagged::new(&[])),
    Case::new(Title::new("a parent without mod.rs"), &[OUTER_FILE, INNER, REACHING], Flagged::new(&[])),
    Case::new(Title::new("users inside the boundary"), &[FAMILY, INNER, FIRST, SECOND], Flagged::new(&[])),
];

#[test]
fn modules_that_belong_inside_another_are_flagged() {
    assert_eq!(Sources::mismatches::<ModuleNesting>(&CASES), Lines::default());
}
