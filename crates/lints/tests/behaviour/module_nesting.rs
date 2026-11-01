use lints::ModuleNesting;

use crate::common::{Case, Source, Sources};

const ROOT: Source = Source::new("crates/a/src/lib.rs", "mod parent;\nmod helper;\n");
const PARENT: Source = Source::new("crates/a/src/parent.rs", "use crate::helper::Tool;\n");
const SECOND_PARENT: Source = Source::new("crates/a/src/other.rs", "use crate::helper::Tool;\n");
const HELPER: Source = Source::new("crates/a/src/helper.rs", "pub struct Tool;\n");
const WIDE: Source = Source::new("crates/a/src/wide.rs", "pub use crate::x::{A, B, C, D, E, F, G, H, I};\n");
const NARROW: Source = Source::new("crates/a/src/narrow.rs", "pub use crate::x::{A, B};\n");
const OUTER: Source = Source::new("crates/a/src/outer/mod.rs", "pub mod inner;\n");
const INNER: Source = Source::new("crates/a/src/outer/inner.rs", "pub struct Thing;\n");
const REACHING: Source = Source::new("crates/a/src/user.rs", "use crate::outer::inner::Thing;\n");
const THROUGH: Source = Source::new("crates/a/src/user.rs", "use crate::outer::Thing;\n");
const OTHER_THROUGH: Source = Source::new("crates/a/src/other.rs", "use crate::outer::Thing;\n");

const HIDDEN_WIDE: Source =
    Source::new("crates/a/src/wide.rs", "pub(crate) use crate::x::{A, B, C, D, E, F, G, H, I};\n");

const CASES: [Case; 7] = [
    Case { name: "used by one sibling", sources: &[ROOT, PARENT, HELPER], flagged: &["crates/a/src/helper.rs:1"] },
    Case { name: "used by two siblings", sources: &[ROOT, PARENT, SECOND_PARENT, HELPER], flagged: &[] },
    Case { name: "wide re-exports", sources: &[WIDE], flagged: &["crates/a/src/wide.rs:1"] },
    Case { name: "narrow re-exports", sources: &[NARROW], flagged: &[] },
    Case { name: "reaching past mod.rs", sources: &[OUTER, INNER, REACHING], flagged: &["crates/a/src/user.rs:1"] },
    Case { name: "through the parent", sources: &[OUTER, INNER, THROUGH, OTHER_THROUGH], flagged: &[] },
    Case { name: "crate-only re-exports", sources: &[HIDDEN_WIDE], flagged: &[] },
];

#[test]
fn modules_that_belong_inside_another_are_flagged() {
    assert_eq!(Sources::mismatches::<ModuleNesting>(&CASES), Vec::<String>::new());
}
