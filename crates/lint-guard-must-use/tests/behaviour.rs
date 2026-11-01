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

laws::table!(unmarked_guards_are_flagged, Sources::flagged::<GuardMustUse>, [
    [Source::new("a.rs", UNMARKED_ENUM)] => ["a.rs:1"],
    [Source::new("a.rs", GENERIC)] => ["a.rs:1"],
    [Source::new("a.rs", QUALIFIED)] => ["a.rs:1"],
    [Source::new("a.rs", OTHER_ATTRIBUTE)] => ["a.rs:2"],
    [Source::new("a.rs", IN_MODULE)] => ["a.rs:2"],
    [Source::new("a.rs", DROP_IMPL), Source::new("b.rs", "struct Lock;\n")] => ["b.rs:1"],
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
]);
