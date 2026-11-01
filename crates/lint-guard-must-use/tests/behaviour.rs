use lint_fixtures::{Located, Source, Sources, Text};
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
    [Source::new(Located::new("a.rs"), Text::new(UNMARKED_ENUM))] => ["a.rs:1"],
    [Source::new(Located::new("a.rs"), Text::new(GENERIC))] => ["a.rs:1"],
    [Source::new(Located::new("a.rs"), Text::new(QUALIFIED))] => ["a.rs:1"],
    [Source::new(Located::new("a.rs"), Text::new(OTHER_ATTRIBUTE))] => ["a.rs:2"],
    [Source::new(Located::new("a.rs"), Text::new(IN_MODULE))] => ["a.rs:2"],
    [Source::new(Located::new("crates/x/src/a.rs"), Text::new(DROP_IMPL)), Source::new(Located::new("crates/x/src/b.rs"), Text::new(PLAIN))] => ["crates/x/src/b.rs:1"],
    [Source::new(Located::new("a.rs"), Text::new(ALIASED))] => ["a.rs:1"],
    [Source::new(Located::new("a.rs"), Text::new(UNION))] => ["a.rs:1"],
    [Source::new(Located::new("a.rs"), Text::new(OTHER_CONDITIONAL))] => ["a.rs:2"],
]);

laws::table!(marked_guards_and_other_types_are_not, Sources::flagged::<GuardMustUse>, [
    [Source::new(Located::new("a.rs"), Text::new(MARKED_REASON))] => NONE,
    [Source::new(Located::new("a.rs"), Text::new(INHERENT))] => NONE,
    [Source::new(Located::new("a.rs"), Text::new(DROP_OF_REFERENCE))] => NONE,
    [Source::new(Located::new("a.rs"), Text::new(TEST_TYPE))] => NONE,
    [Source::new(Located::new("a.rs"), Text::new(TEST_ENUM))] => NONE,
    [Source::new(Located::new("a.rs"), Text::new(TEST_IMPL))] => NONE,
    [Source::new(Located::new("a.rs"), Text::new(TEST_MODULE))] => NONE,
    [Source::new(Located::new("x/tests/a.rs"), Text::new(UNMARKED))] => NONE,
    [Source::new(Located::new("a.rs"), Text::new(CONDITIONAL_MARK))] => NONE,
    [Source::new(Located::new("crates/x/src/lib.rs"), Text::new("mod a;\nmod b;\n")), Source::new(Located::new("crates/x/src/a.rs"), Text::new(MARKED_HERE)), Source::new(Located::new("crates/x/src/b.rs"), Text::new(PLAIN))] => NONE,
    [Source::new(Located::new("crates/one/src/lib.rs"), Text::new(MARKED_HERE)), Source::new(Located::new("crates/two/src/lib.rs"), Text::new(PLAIN))] => NONE,
]);

laws::table!(a_guard_in_another_crate_is_not_this_one, Sources::flagged::<GuardMustUse>, [
    [Source::new(Located::new("crates/one/src/lib.rs"), Text::new(DROP_IMPL)), Source::new(Located::new("crates/two/src/lib.rs"), Text::new(PLAIN))] => NONE,
    [Source::new(Located::new("crates/one/src/lib.rs"), Text::new(UNMARKED)), Source::new(Located::new("crates/two/src/lib.rs"), Text::new(PLAIN))] => ["crates/one/src/lib.rs:1"],
]);
