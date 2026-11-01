use lint_module_nesting::ModuleNesting;

use lint_fixtures::{Case, Source, Sources};

const ROOT: Source = Source::new("crates/a/src/lib.rs", "mod parent;\nmod helper;\n");
const PARENT: Source = Source::new("crates/a/src/parent.rs", "use crate::helper::Tool;\n");
const SECOND_PARENT: Source = Source::new("crates/a/src/other.rs", "use crate::helper::Tool;\n");
const HELPER: Source = Source::new("crates/a/src/helper.rs", "pub struct Tool;\n");
const OUTER: Source = Source::new("crates/a/src/outer/mod.rs", "pub mod inner;\n");
const INNER: Source = Source::new("crates/a/src/outer/inner.rs", "pub struct Thing;\n");
const REACHING: Source = Source::new("crates/a/src/user.rs", "use crate::outer::inner::Thing;\n");
const THROUGH: Source = Source::new("crates/a/src/user.rs", "use crate::outer::Thing;\n");
const OTHER_THROUGH: Source = Source::new("crates/a/src/other.rs", "use crate::outer::Thing;\n");

const HIDDEN_WIDE: Source =
    Source::new("crates/a/src/wide.rs", "pub(crate) use crate::x::{A, B, C, D, E, F, G, H, I};\n");

const OUTER_FILE: Source = Source::new("crates/a/src/outer.rs", "pub mod inner;\n");
const FAMILY: Source = Source::new("crates/a/src/outer/mod.rs", "pub mod inner;\npub mod first;\npub mod second;\n");
const FIRST: Source = Source::new("crates/a/src/outer/first.rs", "use crate::outer::inner::Thing;\n");
const SECOND: Source = Source::new("crates/a/src/outer/second.rs", "use crate::outer::inner::Thing;\n");

const CASES: [Case; 5] = [
    Case::new("used by two siblings", &[ROOT, PARENT, SECOND_PARENT, HELPER], &[]),
    Case::new("through the parent", &[OUTER, INNER, THROUGH, OTHER_THROUGH], &[]),
    Case::new("crate-only re-exports", &[HIDDEN_WIDE], &[]),
    Case::new("a parent without mod.rs", &[OUTER_FILE, INNER, REACHING], &[]),
    Case::new("users inside the boundary", &[FAMILY, INNER, FIRST, SECOND], &[]),
];

#[test]
fn modules_that_belong_inside_another_are_flagged() {
    assert_eq!(Sources::mismatches::<ModuleNesting>(&CASES), Vec::<String>::new());
}
