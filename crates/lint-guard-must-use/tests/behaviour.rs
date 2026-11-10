use lint_fixtures::{Lines, Located, Source, Sources, Text};
use lint_guard_must_use::GuardMustUse;

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
const MARK_AS_ARGUMENT: &str =
    "#[derive(Debug, must_use)]\npub struct Lock;\nimpl Drop for Lock {\n    fn drop(&mut self) {}\n}\n";
const CONDITIONAL_MARK: &str =
    "#[cfg_attr(all(), must_use)]\npub struct Lock;\nimpl Drop for Lock {\n    fn drop(&mut self) {}\n}\n";
const ALIASED: &str = "pub struct Lock;\ntype Held = Lock;\nimpl Drop for Held {\n    fn drop(&mut self) {}\n}\n";
const UNION: &str = "pub union Lock {\n    a: u8,\n}\nimpl Drop for Lock {\n    fn drop(&mut self) {}\n}\n";
const MARKED_HERE: &str = "#[must_use]\npub struct Lock;\nimpl Drop for Lock {\n    fn drop(&mut self) {}\n}\n";

laws::table!(unmarked_guards_are_flagged, Sources::flagged::<GuardMustUse>, [
    [Source::new(Located::new("a.rs"), Text::new(UNMARKED_ENUM))] => Lines::from(["a.rs:1"]),
    [Source::new(Located::new("a.rs"), Text::new(GENERIC))] => Lines::from(["a.rs:1"]),
    [Source::new(Located::new("a.rs"), Text::new(QUALIFIED))] => Lines::from(["a.rs:1"]),
    [Source::new(Located::new("a.rs"), Text::new(OTHER_ATTRIBUTE))] => Lines::from(["a.rs:2"]),
    [Source::new(Located::new("a.rs"), Text::new(IN_MODULE))] => Lines::from(["a.rs:2"]),
    [Source::new(Located::new("crates/x/src/a.rs"), Text::new(DROP_IMPL)), Source::new(Located::new("crates/x/src/b.rs"), Text::new(PLAIN))] => Lines::from(["crates/x/src/b.rs:1"]),
    [Source::new(Located::new("a.rs"), Text::new(ALIASED))] => Lines::from(["a.rs:1"]),
    [Source::new(Located::new("a.rs"), Text::new(UNION))] => Lines::from(["a.rs:1"]),
    [Source::new(Located::new("a.rs"), Text::new(OTHER_CONDITIONAL))] => Lines::from(["a.rs:2"]),
    [Source::new(Located::new("a.rs"), Text::new(MARK_AS_ARGUMENT))] => Lines::from(["a.rs:2"]),
]);

laws::table!(marked_guards_and_other_types_are_not, Sources::flagged::<GuardMustUse>, [
    [Source::new(Located::new("a.rs"), Text::new(MARKED_REASON))] => Lines::default(),
    [Source::new(Located::new("a.rs"), Text::new(INHERENT))] => Lines::default(),
    [Source::new(Located::new("a.rs"), Text::new(DROP_OF_REFERENCE))] => Lines::default(),
    [Source::new(Located::new("a.rs"), Text::new(TEST_TYPE))] => Lines::default(),
    [Source::new(Located::new("a.rs"), Text::new(TEST_ENUM))] => Lines::default(),
    [Source::new(Located::new("a.rs"), Text::new(TEST_IMPL))] => Lines::default(),
    [Source::new(Located::new("a.rs"), Text::new(TEST_MODULE))] => Lines::default(),
    [Source::new(Located::new("x/tests/a.rs"), Text::new(UNMARKED))] => Lines::default(),
    [Source::new(Located::new("a.rs"), Text::new(CONDITIONAL_MARK))] => Lines::default(),
    [Source::new(Located::new("crates/x/src/lib.rs"), Text::new("mod a;\nmod b;\n")), Source::new(Located::new("crates/x/src/a.rs"), Text::new(MARKED_HERE)), Source::new(Located::new("crates/x/src/b.rs"), Text::new(PLAIN))] => Lines::default(),
    [Source::new(Located::new("crates/one/src/lib.rs"), Text::new(MARKED_HERE)), Source::new(Located::new("crates/two/src/lib.rs"), Text::new(PLAIN))] => Lines::default(),
]);

laws::table!(a_guard_in_another_crate_is_not_this_one, Sources::flagged::<GuardMustUse>, [
    [Source::new(Located::new("crates/one/src/lib.rs"), Text::new(DROP_IMPL)), Source::new(Located::new("crates/two/src/lib.rs"), Text::new(PLAIN))] => Lines::default(),
    [Source::new(Located::new("crates/one/src/lib.rs"), Text::new(UNMARKED)), Source::new(Located::new("crates/two/src/lib.rs"), Text::new(PLAIN))] => Lines::from(["crates/one/src/lib.rs:1"]),
]);
