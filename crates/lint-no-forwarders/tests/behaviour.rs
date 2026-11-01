use lint_no_forwarders::NoForwarders;

use lint_fixtures::{Case, Source, Sources};

const METHOD: &str = "struct Inner;\nimpl Inner {\n    fn work(&self) -> u8 { 1 }\n}\nstruct Outer { inner: Inner }\nimpl Outer {\n    fn work(&self) -> u8 {\n        self.inner.work()\n    }\n}\n";
const OWN_LOGIC: &str = "struct Inner;\nimpl Inner {\n    fn work(&self) -> u8 { 1 }\n}\nstruct Outer { inner: Inner }\nimpl Outer {\n    fn work(&self) -> u8 {\n        self.inner.work() + 1\n    }\n}\n";
const FOREIGN: &str = "struct Outer { inner: Vec<u8> }\nimpl Outer {\n    fn size(&self) -> usize {\n        self.inner.len()\n    }\n}\n";
const ASSOCIATED: &str = "struct Inner;\nimpl Inner {\n    fn make() -> u8 { 1 }\n}\nstruct Outer;\nimpl Outer {\n    fn make() -> u8 {\n        Inner::make()\n    }\n}\n";
const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    struct Inner;\n    impl Inner {\n        fn work(&self) -> u8 { 1 }\n    }\n    struct Outer { inner: Inner }\n    impl Outer {\n        fn work(&self) -> u8 {\n            self.inner.work()\n        }\n    }\n}\n";

const CASES: [Case; 5] = [
    Case::new("method", &[Source::new("a.rs", METHOD)], &["a.rs:7"]),
    Case::new("own logic", &[Source::new("a.rs", OWN_LOGIC)], &[]),
    Case::new("foreign type", &[Source::new("a.rs", FOREIGN)], &[]),
    Case::new("associated", &[Source::new("a.rs", ASSOCIATED)], &["a.rs:7"]),
    Case::new("test module", &[Source::new("a.rs", TEST_MODULE)], &[]),
];

#[test]
fn methods_that_only_forward_to_another_type_of_ours_are_flagged() {
    assert_eq!(Sources::mismatches::<NoForwarders>(&CASES), Vec::<String>::new());
}
