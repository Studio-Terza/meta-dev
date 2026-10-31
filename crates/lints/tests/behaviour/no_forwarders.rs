use lints::NoForwarders;

use crate::common::{Case, Source, Sources};

const METHOD: &str = "struct Inner;\nimpl Inner {\n    fn work(&self) -> u8 { 1 }\n}\nstruct Outer { inner: Inner }\nimpl Outer {\n    fn work(&self) -> u8 {\n        self.inner.work()\n    }\n}\n";
const OWN_LOGIC: &str = "struct Inner;\nimpl Inner {\n    fn work(&self) -> u8 { 1 }\n}\nstruct Outer { inner: Inner }\nimpl Outer {\n    fn work(&self) -> u8 {\n        self.inner.work() + 1\n    }\n}\n";
const FOREIGN: &str = "struct Outer { inner: Vec<u8> }\nimpl Outer {\n    fn size(&self) -> usize {\n        self.inner.len()\n    }\n}\n";
const ASSOCIATED: &str = "struct Inner;\nimpl Inner {\n    fn make() -> u8 { 1 }\n}\nstruct Outer;\nimpl Outer {\n    fn make() -> u8 {\n        Inner::make()\n    }\n}\n";
const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    struct Inner;\n    impl Inner {\n        fn work(&self) -> u8 { 1 }\n    }\n    struct Outer { inner: Inner }\n    impl Outer {\n        fn work(&self) -> u8 {\n            self.inner.work()\n        }\n    }\n}\n";

const CASES: [Case; 5] = [
    Case { name: "method", sources: &[Source::new("a.rs", METHOD)], flagged: &["a.rs:7"] },
    Case { name: "own logic", sources: &[Source::new("a.rs", OWN_LOGIC)], flagged: &[] },
    Case { name: "foreign type", sources: &[Source::new("a.rs", FOREIGN)], flagged: &[] },
    Case { name: "associated", sources: &[Source::new("a.rs", ASSOCIATED)], flagged: &["a.rs:7"] },
    Case { name: "test module", sources: &[Source::new("a.rs", TEST_MODULE)], flagged: &[] },
];

#[test]
fn methods_that_only_forward_to_another_type_of_ours_are_flagged() {
    assert_eq!(Sources::mismatches::<NoForwarders>(&CASES), Vec::<String>::new());
}
