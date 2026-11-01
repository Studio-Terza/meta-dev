use lint_fixtures::{Source, Sources};
use lint_paired_apis::PairedApis;

const NONE: [&str; 0] = [];
const START_STOP: &str = "pub struct A;\nimpl A {\n    pub fn start(&self) {}\n    pub fn stop(&self) {}\n}\n";
const LOCK_UNLOCK: &str = "pub struct A;\nimpl A {\n    pub fn lock(&self) {}\n    pub fn unlock(&self) {}\n}\n";
const OPEN_CLOSE: &str = "pub struct A;\nimpl A {\n    pub fn close(&self) {}\n    pub fn open(&self) {}\n}\n";
const CLAIM_RELEASE: &str = "pub struct A;\nimpl A {\n    pub fn claim(&self) {}\n    pub fn release(&self) {}\n}\n";
const ACQUIRE_RELEASE: &str =
    "pub struct A;\nimpl A {\n    pub fn acquire(&self) {}\n    pub fn release(&self) {}\n}\n";
const BEGIN_END: &str = "pub struct A;\nimpl A {\n    pub fn begin(&self) {}\n    pub fn end(&self) {}\n}\n";
const SUFFIXED: &str =
    "pub struct A;\nimpl A {\n    pub fn start_server(&self) {}\n    pub fn stop_server(&self) {}\n}\n";
const IN_TRAIT: &str = "trait Port {\n    fn open(&self);\n    fn close(&self);\n}\n";
const IN_MODULE: &str =
    "mod inner {\n    impl A {\n        pub fn lock(&self) {}\n        pub fn unlock(&self) {}\n    }\n}\n";
const ALONE: &str = "pub struct A;\nimpl A {\n    pub fn start(&self) {}\n    pub fn close(&self) {}\n}\n";
const OTHER_SUFFIX: &str =
    "pub struct A;\nimpl A {\n    pub fn start_server(&self) {}\n    pub fn stop_client(&self) {}\n}\n";
const HALF_PRIVATE: &str = "pub struct A;\nimpl A {\n    pub fn start(&self) {}\n    fn stop(&self) {}\n}\n";
const TWO_BLOCKS: &str = "impl A {\n    pub fn start(&self) {}\n}\nimpl B {\n    pub fn stop(&self) {}\n}\n";
const SPLIT_BLOCKS: &str = "impl A {\n    pub fn start(&self) {}\n}\nimpl A {\n    pub fn stop(&self) {}\n}\n";
const TRAIT_IMPL: &str = "impl Port for A {\n    fn open(&self) {}\n    fn close(&self) {}\n}\n";
const TRAIT_CONSTS: &str = "trait Port {\n    const OPEN: u8;\n    fn open(&self);\n}\n";
const IMPL_CONSTS: &str = "impl A {\n    const CLOSE: u8 = 1;\n    pub fn open(&self) {}\n}\n";
const TEST_IMPL: &str = "#[cfg(test)]\nimpl A {\n    pub fn open(&self) {}\n    pub fn close(&self) {}\n}\n";
const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    impl A {\n        pub fn open(&self) {}\n        pub fn close(&self) {}\n    }\n}\n";
const TEST_TRAIT: &str = "#[cfg(test)]\ntrait Port {\n    fn open(&self);\n    fn close(&self);\n}\n";

laws::table!(paired_methods_are_listed, Sources::flagged::<PairedApis>, [
    [Source::new("a.rs", START_STOP)] => ["a.rs:3"],
    [Source::new("a.rs", LOCK_UNLOCK)] => ["a.rs:3"],
    [Source::new("a.rs", OPEN_CLOSE)] => ["a.rs:4"],
    [Source::new("a.rs", CLAIM_RELEASE)] => ["a.rs:3"],
    [Source::new("a.rs", ACQUIRE_RELEASE)] => ["a.rs:3"],
    [Source::new("a.rs", BEGIN_END)] => ["a.rs:3"],
    [Source::new("a.rs", SUFFIXED)] => ["a.rs:3"],
    [Source::new("a.rs", IN_TRAIT)] => ["a.rs:2"],
    [Source::new("a.rs", IN_MODULE)] => ["a.rs:3"],
    [Source::new("a.rs", SPLIT_BLOCKS)] => ["a.rs:2"],
    [Source::new("crates/x/src/lib.rs", "mod a;\nmod b;\n"), Source::new("crates/x/src/a.rs", "impl A {\n    pub fn open(&self) {}\n}\n"), Source::new("crates/x/src/b.rs", "impl A {\n    pub fn close(&self) {}\n}\n")] => ["crates/x/src/a.rs:2"],
]);

laws::table!(unpaired_methods_are_not, Sources::flagged::<PairedApis>, [
    [Source::new("a.rs", ALONE)] => NONE,
    [Source::new("a.rs", OTHER_SUFFIX)] => NONE,
    [Source::new("a.rs", HALF_PRIVATE)] => NONE,
    [Source::new("a.rs", TWO_BLOCKS)] => NONE,
    [Source::new("crates/one/src/lib.rs", "impl A {\n    pub fn open(&self) {}\n}\n"), Source::new("crates/two/src/lib.rs", "impl A {\n    pub fn close(&self) {}\n}\n")] => NONE,
    [Source::new("a.rs", TRAIT_IMPL)] => NONE,
    [Source::new("a.rs", TRAIT_CONSTS)] => NONE,
    [Source::new("a.rs", IMPL_CONSTS)] => NONE,
    [Source::new("a.rs", TEST_IMPL)] => NONE,
    [Source::new("a.rs", TEST_MODULE)] => NONE,
    [Source::new("a.rs", TEST_TRAIT)] => NONE,
    [Source::new("x/tests/a.rs", START_STOP)] => NONE,
]);

#[test]
fn the_message_says_to_write_a_guard() {
    let shown = Sources::report::<PairedApis>(&[Source::new("a.rs", SUFFIXED)]).to_string();
    assert!(["guard", "fn start_server pairs with fn stop_server"].iter().all(|word| shown.contains(word)));
}
