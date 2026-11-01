use lint_fixtures::{Source, Sources};
use lint_guard_must_use::GuardMustUse;

const NONE: [&str; 0] = [];
const DROP_IMPL: &str = "impl Drop for Lock {\n    fn drop(&mut self) {}\n}\n";
const UNMARKED: &str = "pub struct Lock;\nimpl Drop for Lock {\n    fn drop(&mut self) {}\n}\n";
const UNMARKED_ENUM: &str = "enum Lock {\n    Held,\n}\nimpl Drop for Lock {\n    fn drop(&mut self) {}\n}\n";
const GENERIC: &str = "struct Lock<'a, T>(&'a T);\nimpl<T> Drop for Lock<'_, T> {\n    fn drop(&mut self) {}\n}\n";
const QUALIFIED: &str = "struct Lock;\nimpl std::ops::Drop for inner::Lock {\n    fn drop(&mut self) {}\n}\n";
const MARKED_REASON: &str = "#[must_use = \"held until dropped\"]\nenum Lock {\n    Held,\n}\nimpl Drop for Lock {\n    fn drop(&mut self) {}\n}\n";
const OTHER_ATTRIBUTE: &str = "#[derive(Debug)]\nstruct Lock;\nimpl Drop for Lock {\n    fn drop(&mut self) {}\n}\n";
const IN_MODULE: &str =
    "mod inner {\n    struct Lock;\n    impl Drop for Lock {\n        fn drop(&mut self) {}\n    }\n}\n";
const INHERENT: &str = "struct Lock;\nimpl Lock {\n    fn drop(&mut self) {}\n}\n";
const DROP_OF_REFERENCE: &str = "struct Lock;\nimpl Drop for &Lock {\n    fn drop(&mut self) {}\n}\n";
const TEST_TYPE: &str = "impl Drop for Lock {\n    fn drop(&mut self) {}\n}\n#[cfg(test)]\nstruct Lock;\n";
const TEST_ENUM: &str = "impl Drop for Lock {\n    fn drop(&mut self) {}\n}\n#[cfg(test)]\nenum Lock {}\n";
const TEST_IMPL: &str = "struct Lock;\n#[cfg(test)]\nimpl Drop for Lock {\n    fn drop(&mut self) {}\n}\n";
const TEST_MODULE: &str =
    "#[cfg(test)]\nmod tests {\n    struct Lock;\n    impl Drop for Lock {\n        fn drop(&mut self) {}\n    }\n}\n";

const OTHER_CONDITIONAL: &str =
    "#[cfg_attr(all(), derive(Debug))]\npub struct Lock;\nimpl Drop for Lock {\n    fn drop(&mut self) {}\n}\n";
const PLAIN: &str = "pub struct Lock;\n";
const CONDITIONAL_MARK: &str =
    "#[cfg_attr(all(), must_use)]\npub struct Lock;\nimpl Drop for Lock {\n    fn drop(&mut self) {}\n}\n";
const ALIASED: &str = "pub struct Lock;\ntype Held = Lock;\nimpl Drop for Held {\n    fn drop(&mut self) {}\n}\n";
const UNION: &str = "pub union Lock {\n    a: u8,\n}\nimpl Drop for Lock {\n    fn drop(&mut self) {}\n}\n";
const MARKED_HERE: &str = "#[must_use]\npub struct Lock;\nimpl Drop for Lock {\n    fn drop(&mut self) {}\n}\n";

laws::table!(unmarked_guards_are_flagged, Sources::flagged::<GuardMustUse>, [
    [Source::new("a.rs", UNMARKED_ENUM)] => ["a.rs:1"],
    [Source::new("a.rs", GENERIC)] => ["a.rs:1"],
    [Source::new("a.rs", QUALIFIED)] => ["a.rs:1"],
    [Source::new("a.rs", OTHER_ATTRIBUTE)] => ["a.rs:2"],
    [Source::new("a.rs", IN_MODULE)] => ["a.rs:2"],
    [Source::new("crates/x/src/a.rs", DROP_IMPL), Source::new("crates/x/src/b.rs", PLAIN)] => ["crates/x/src/b.rs:1"],
    [Source::new("a.rs", ALIASED)] => ["a.rs:1"],
    [Source::new("a.rs", UNION)] => ["a.rs:1"],
    [Source::new("a.rs", OTHER_CONDITIONAL)] => ["a.rs:2"],
]);

laws::table!(marked_guards_and_other_types_are_not, Sources::flagged::<GuardMustUse>, [
    [Source::new("a.rs", MARKED_REASON)] => NONE,
    [Source::new("a.rs", INHERENT)] => NONE,
    [Source::new("a.rs", DROP_OF_REFERENCE)] => NONE,
    [Source::new("a.rs", TEST_TYPE)] => NONE,
    [Source::new("a.rs", TEST_ENUM)] => NONE,
    [Source::new("a.rs", TEST_IMPL)] => NONE,
    [Source::new("a.rs", TEST_MODULE)] => NONE,
    [Source::new("x/tests/a.rs", UNMARKED)] => NONE,
    [Source::new("a.rs", CONDITIONAL_MARK)] => NONE,
    [Source::new("crates/x/src/lib.rs", "mod a;\nmod b;\n"), Source::new("crates/x/src/a.rs", MARKED_HERE), Source::new("crates/x/src/b.rs", PLAIN)] => NONE,
    [Source::new("crates/one/src/lib.rs", MARKED_HERE), Source::new("crates/two/src/lib.rs", PLAIN)] => NONE,
]);

laws::table!(a_guard_in_another_crate_is_not_this_one, Sources::flagged::<GuardMustUse>, [
    [Source::new("crates/one/src/lib.rs", DROP_IMPL), Source::new("crates/two/src/lib.rs", PLAIN)] => NONE,
    [Source::new("crates/one/src/lib.rs", UNMARKED), Source::new("crates/two/src/lib.rs", PLAIN)] => ["crates/one/src/lib.rs:1"],
]);
