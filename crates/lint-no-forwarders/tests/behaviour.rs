use lint_no_forwarders::NoForwarders;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

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
    Case::new(
        Title::new("test module"),
        &[Source::new(Located::new("a.rs"), Text::new(TEST_MODULE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a parameter passed on"),
        &[Source::new(Located::new("a.rs"), Text::new(ARGUMENT))],
        Flagged::new(&["a.rs:8"]),
    ),
    Case::new(
        Title::new("a borrowed parameter passed on"),
        &[Source::new(Located::new("a.rs"), Text::new(BORROWED_ARGUMENT))],
        Flagged::new(&["a.rs:8"]),
    ),
    Case::new(
        Title::new("a field of a parameter passed on"),
        &[Source::new(Located::new("a.rs"), Text::new(FIELD_ARGUMENT))],
        Flagged::new(&["a.rs:8"]),
    ),
    Case::new(
        Title::new("a constant argument"),
        &[Source::new(Located::new("a.rs"), Text::new(CONSTANT_ARGUMENT))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("own method"),
        &[Source::new(Located::new("a.rs"), Text::new(OWN_METHOD))],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(
        Title::new("method of a parameter"),
        &[Source::new(Located::new("a.rs"), Text::new(PARAMETER))],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(
        Title::new("method of a borrowed parameter"),
        &[Source::new(Located::new("a.rs"), Text::new(BORROWED_PARAMETER))],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(
        Title::new("method of a foreign parameter"),
        &[Source::new(Located::new("a.rs"), Text::new(FOREIGN_PARAMETER))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a pattern parameter"),
        &[Source::new(Located::new("a.rs"), Text::new(PATTERN_PARAMETER))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("field of a parameter"),
        &[Source::new(Located::new("a.rs"), Text::new(FIELD_OF_A_PARAMETER))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("associated call passing a parameter"),
        &[Source::new(Located::new("a.rs"), Text::new(ASSOCIATED_ARGUMENT))],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(
        Title::new("associated call passing a constant"),
        &[Source::new(Located::new("a.rs"), Text::new(ASSOCIATED_CONSTANT))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("own associated call"),
        &[Source::new(Located::new("a.rs"), Text::new(OWN_ASSOCIATED))],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(
        Title::new("a variant constructor"),
        &[Source::new(Located::new("a.rs"), Text::new(VARIANT))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a foreign owner"),
        &[Source::new(Located::new("a.rs"), Text::new(FOREIGN_OWNER))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("an enum owner"),
        &[Source::new(Located::new("a.rs"), Text::new(ENUM_OWNER))],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(
        Title::new("a trait owner"),
        &[Source::new(Located::new("a.rs"), Text::new(TRAIT_OWNER))],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(
        Title::new("a generic field"),
        &[Source::new(Located::new("a.rs"), Text::new(GENERIC_FIELD))],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(
        Title::new("a generic field of a foreign type"),
        &[Source::new(Located::new("a.rs"), Text::new(FOREIGN_GENERIC_FIELD))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a plain field of a generic struct"),
        &[Source::new(Located::new("a.rs"), Text::new(PLAIN_FIELD_OF_A_GENERIC))],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(
        Title::new("a borrowed generic field"),
        &[Source::new(Located::new("a.rs"), Text::new(BORROWED_GENERIC_FIELD))],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(
        Title::new("inline module"),
        &[Source::new(Located::new("a.rs"), Text::new(INLINE))],
        Flagged::new(&["a.rs:5"]),
    ),
];

#[test]
fn methods_that_only_forward_to_another_type_of_ours_are_flagged() {
    assert_eq!(Sources::mismatches::<NoForwarders>(&CASES), Lines::default());
}
