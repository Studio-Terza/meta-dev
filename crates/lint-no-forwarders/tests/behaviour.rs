use lint_no_forwarders::NoForwarders;

use lint_fixtures::{Case, Source, Sources};

const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    struct Inner;\n    impl Inner {\n        fn work(&self) -> u8 { 1 }\n    }\n    struct Outer { inner: Inner }\n    impl Outer {\n        fn work(&self) -> u8 {\n            self.inner.work()\n        }\n    }\n}\n";

const ARGUMENT: &str = "struct Inner;\nimpl Inner {\n    fn work(&self, x: u8) -> u8 { x }\n    fn make() -> u8 { 1 }\n}\nstruct Outer { inner: Inner }\nimpl Outer {\n    fn pass(&self, x: u8, y: Pair) -> u8 {\n        self.inner.work(x)\n    }\n}\n";
const BORROWED_ARGUMENT: &str = "struct Inner;\nimpl Inner {\n    fn work(&self, x: u8) -> u8 { x }\n    fn make() -> u8 { 1 }\n}\nstruct Outer { inner: Inner }\nimpl Outer {\n    fn pass(&self, x: u8, y: Pair) -> u8 {\n        self.inner.work(&x)\n    }\n}\n";
const FIELD_ARGUMENT: &str = "struct Inner;\nimpl Inner {\n    fn work(&self, x: u8) -> u8 { x }\n    fn make() -> u8 { 1 }\n}\nstruct Outer { inner: Inner }\nimpl Outer {\n    fn pass(&self, x: u8, y: Pair) -> u8 {\n        self.inner.work(y.part)\n    }\n}\n";
const CONSTANT_ARGUMENT: &str = "struct Inner;\nimpl Inner {\n    fn work(&self, x: u8) -> u8 { x }\n    fn make() -> u8 { 1 }\n}\nstruct Outer { inner: Inner }\nimpl Outer {\n    fn pass(&self, x: u8, y: Pair) -> u8 {\n        self.inner.work(1)\n    }\n}\n";
const OWN_METHOD: &str = "struct Outer;\nimpl Outer {\n    fn run(&self) -> u8 { 1 }\n    fn pass(&self) -> u8 {\n        self.run()\n    }\n}\n";
const PARAMETER: &str = "struct Inner;\nstruct Outer;\nimpl Outer {\n    fn pass(&self, inner: Inner) -> u8 {\n        inner.work()\n    }\n}\n";
const BORROWED_PARAMETER: &str = "struct Inner;\nstruct Outer;\nimpl Outer {\n    fn pass(&self, inner: &Inner) -> u8 {\n        inner.work()\n    }\n}\n";
const FOREIGN_PARAMETER: &str =
    "struct Outer;\nimpl Outer {\n    fn pass(&self, inner: Foreign) -> u8 {\n        inner.work()\n    }\n}\n";
const PATTERN_PARAMETER: &str = "struct Inner;\nimpl Inner {\n    fn work(&self, x: u8) -> u8 { x }\n    fn make() -> u8 { 1 }\n}\nstruct Outer { inner: Inner }\nimpl Outer {\n    fn pass(&self, x: u8, (a, b): Pair) -> u8 {\n        self.inner.work(a)\n    }\n}\n";
const FIELD_OF_A_PARAMETER: &str = "struct Inner;\nstruct Outer { inner: Inner }\nimpl Outer {\n    fn pass(&self, other: Outer) -> u8 {\n        other.inner.work()\n    }\n}\n";
const ASSOCIATED_ARGUMENT: &str =
    "struct Inner;\nstruct Outer;\nimpl Outer {\n    fn pass(x: u8) -> u8 {\n        Inner::make(x)\n    }\n}\n";
const ASSOCIATED_CONSTANT: &str =
    "struct Inner;\nstruct Outer;\nimpl Outer {\n    fn pass() -> u8 {\n        Inner::make(1)\n    }\n}\n";
const OWN_ASSOCIATED: &str =
    "struct Outer;\nimpl Outer {\n    fn make() -> u8 { 1 }\n    fn pass() -> u8 {\n        Self::make()\n    }\n}\n";
const VARIANT: &str =
    "struct Inner;\nstruct Outer;\nimpl Outer {\n    fn pass(x: u8) -> Inner {\n        Inner::Wrap(x)\n    }\n}\n";
const FOREIGN_OWNER: &str = "struct Outer;\nimpl Outer {\n    fn pass() -> u8 {\n        Foreign::make()\n    }\n}\n";
const ENUM_OWNER: &str =
    "enum Inner {}\nstruct Outer;\nimpl Outer {\n    fn pass() -> u8 {\n        Inner::make()\n    }\n}\n";
const TRAIT_OWNER: &str =
    "trait Inner {}\nstruct Outer;\nimpl Outer {\n    fn pass() -> u8 {\n        Inner::make()\n    }\n}\n";
const GENERIC_FIELD: &str = "struct Inner;\nstruct Outer<T> { inner: T }\nimpl Outer<Inner> {\n    fn pass(&self) -> u8 {\n        self.inner.work()\n    }\n}\n";
const FOREIGN_GENERIC_FIELD: &str = "struct Outer<T> { inner: T }\nimpl Outer<Foreign> {\n    fn pass(&self) -> u8 {\n        self.inner.work()\n    }\n}\n";
const PLAIN_FIELD_OF_A_GENERIC: &str = "struct Inner;\nstruct Outer<T> { inner: Inner, other: T }\nimpl Outer<Foreign> {\n    fn pass(&self) -> u8 {\n        self.inner.work()\n    }\n}\n";
const BORROWED_GENERIC_FIELD: &str = "struct Inner;\nstruct Outer<'text, T> { inner: &'text T }\nimpl Outer<'_, Inner> {\n    fn pass(&self) -> u8 {\n        self.inner.work()\n    }\n}\n";
const INLINE: &str = "mod inner {\n    struct Inner;\n    struct Outer;\n    impl Outer {\n        fn pass(&self, inner: Inner) -> u8 {\n            inner.work()\n        }\n    }\n}\n";

const CASES: [Case; 23] = [
    Case::new("test module", &[Source::new("a.rs", TEST_MODULE)], &[]),
    Case::new("a parameter passed on", &[Source::new("a.rs", ARGUMENT)], &["a.rs:8"]),
    Case::new("a borrowed parameter passed on", &[Source::new("a.rs", BORROWED_ARGUMENT)], &["a.rs:8"]),
    Case::new("a field of a parameter passed on", &[Source::new("a.rs", FIELD_ARGUMENT)], &["a.rs:8"]),
    Case::new("a constant argument", &[Source::new("a.rs", CONSTANT_ARGUMENT)], &[]),
    Case::new("own method", &[Source::new("a.rs", OWN_METHOD)], &["a.rs:4"]),
    Case::new("method of a parameter", &[Source::new("a.rs", PARAMETER)], &["a.rs:4"]),
    Case::new("method of a borrowed parameter", &[Source::new("a.rs", BORROWED_PARAMETER)], &["a.rs:4"]),
    Case::new("method of a foreign parameter", &[Source::new("a.rs", FOREIGN_PARAMETER)], &[]),
    Case::new("a pattern parameter", &[Source::new("a.rs", PATTERN_PARAMETER)], &[]),
    Case::new("field of a parameter", &[Source::new("a.rs", FIELD_OF_A_PARAMETER)], &[]),
    Case::new("associated call passing a parameter", &[Source::new("a.rs", ASSOCIATED_ARGUMENT)], &["a.rs:4"]),
    Case::new("associated call passing a constant", &[Source::new("a.rs", ASSOCIATED_CONSTANT)], &[]),
    Case::new("own associated call", &[Source::new("a.rs", OWN_ASSOCIATED)], &["a.rs:4"]),
    Case::new("a variant constructor", &[Source::new("a.rs", VARIANT)], &[]),
    Case::new("a foreign owner", &[Source::new("a.rs", FOREIGN_OWNER)], &[]),
    Case::new("an enum owner", &[Source::new("a.rs", ENUM_OWNER)], &["a.rs:4"]),
    Case::new("a trait owner", &[Source::new("a.rs", TRAIT_OWNER)], &["a.rs:4"]),
    Case::new("a generic field", &[Source::new("a.rs", GENERIC_FIELD)], &["a.rs:4"]),
    Case::new("a generic field of a foreign type", &[Source::new("a.rs", FOREIGN_GENERIC_FIELD)], &[]),
    Case::new("a plain field of a generic struct", &[Source::new("a.rs", PLAIN_FIELD_OF_A_GENERIC)], &["a.rs:4"]),
    Case::new("a borrowed generic field", &[Source::new("a.rs", BORROWED_GENERIC_FIELD)], &["a.rs:4"]),
    Case::new("inline module", &[Source::new("a.rs", INLINE)], &["a.rs:5"]),
];

#[test]
fn methods_that_only_forward_to_another_type_of_ours_are_flagged() {
    assert_eq!(Sources::mismatches::<NoForwarders>(&CASES), Vec::<String>::new());
}
