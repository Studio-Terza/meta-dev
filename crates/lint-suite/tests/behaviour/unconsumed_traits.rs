use lint_core::Lint;
use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};
use lint_unconsumed_traits::UnconsumedTraits;

const BARE: &str =
    "trait Shape {\n    fn area(&self) -> u32;\n}\nimpl Shape for u8 {\n    fn area(&self) -> u32 { 1 }\n}\n";
const BOUND: &str =
    "trait Shape {\n    fn area(&self) -> u32;\n}\nfn total<T: Shape>(item: &T) -> u32 { item.area() }\n";
const WHERE: &str =
    "trait Shape {\n    fn area(&self) -> u32;\n}\nfn total<T>(item: &T) -> u32 where T: Shape { item.area() }\n";
const DYN: &str = "trait Shape {\n    fn area(&self) -> u32;\n}\nfn total(item: &dyn Shape) -> u32 { item.area() }\n";
const IMPL: &str = "trait Shape {\n    fn area(&self) -> u32;\n}\nfn total(item: impl Shape) -> u32 { item.area() }\n";
const PROVIDED: &str = "trait Shape {\n    fn area(&self) -> u32 { 1 }\n}\n";
const PROVIDED_CALLED: &str = "trait Shape {\n    fn sides(&self) -> u32;\n    fn area(&self) -> u32 { self.sides() }\n}\nfn total(item: &u8) -> u32 { item.area() }\n";
const PROVIDED_PATH: &str = "trait Shape {\n    fn sides() -> u32;\n    fn area() -> u32 { Self::sides() }\n}\nfn total() -> u32 { Square::area() }\n";
const PROVIDED_ONLY_IN_TESTS: &str = "trait Shape {\n    fn sides(&self) -> u32;\n    fn area(&self) -> u32 { self.sides() }\n}\n#[cfg(test)]\nmod tests {\n    fn total(item: &u8) -> u32 { item.area() }\n}\n";
const PROVIDED_IN_MARKUP: &str = "trait Shape {\n    fn sides(&self) -> u32;\n    fn area(&self) -> u32 { self.sides() }\n}\nfn total(item: &u8) -> Node {\n    rsx! { div { width: item.area(), \"wide\" } }\n}\n";
const PROVIDED_NESTED_IN_MARKUP: &str = "trait Shape {\n    fn sides(&self) -> u32;\n    fn area(&self) -> u32 { self.sides() }\n}\nfn total(item: &u8) -> Node {\n    rsx! { div { p { width: item.area(), } } }\n}\n";
const TURBOFISH_IN_MARKUP: &str = "trait Shape {\n    fn sides(&self) -> u32;\n    fn area(&self) -> u32 { self.sides() }\n}\nfn total(item: &u8) -> Node {\n    rsx! { div { width: item.area::<u8>() } }\n}\n";
const RANGE_IN_MARKUP: &str = "trait Shape {\n    fn sides(&self) -> u32;\n    fn area(&self) -> u32 { self.sides() }\n}\nfn total() -> Node {\n    rsx! { for i in 0..area(3) { p {} } }\n}\n";
const INTERPOLATED_IN_MARKUP: &str = "trait Shape {\n    fn sides(&self) -> u32;\n    fn area(&self) -> u32 { self.sides() }\n}\nfn total(item: &u8) -> Node {\n    rsx! { div { class: \"w {item.area()}\" } }\n}\n";
const FIELD_IN_MARKUP: &str = "trait Shape {\n    fn sides(&self) -> u32;\n    fn area(&self) -> u32 { self.sides() }\n}\nfn total(item: &Plan) -> Node {\n    rsx! { div { width: item.area } }\n}\n";
const PATH_IN_MARKUP: &str = "trait Shape {\n    fn sides() -> u32;\n    fn area() -> u32 { Self::sides() }\n}\nfn total() -> Node {\n    rsx! { div { width: Square::area() } }\n}\n";
const LITERAL_IN_JSON: &str = "trait Shape {\n    fn sides(&self) -> u32;\n    fn area(&self) -> u32 { self.sides() }\n}\nfn note() -> Value {\n    json!({ \"hint\": \"{x.area()}\" })\n}\n";
const ESCAPED_BRACES: &str = "trait Shape {\n    fn sides(&self) -> u32;\n    fn area(&self) -> u32 { self.sides() }\n}\nfn note() -> Node {\n    rsx! { div { \"{{item.area()}}\" } }\n}\n";
const LITERAL_IN_ASSERT: &str = "trait Shape {\n    fn sides(&self) -> u32;\n    fn area(&self) -> u32 { self.sides() }\n}\nfn check(label: &str) {\n    assert_eq!(label, \"{item.area()}\");\n}\n";
const LITERAL_IN_PRINT: &str = "trait Shape {\n    fn sides(&self) -> u32;\n    fn area(&self) -> u32 { self.sides() }\n}\nfn show() {\n    println!(\"{}\", \"{item.area()}\");\n}\n";
const JSON_IN_MARKUP: &str = "trait Shape {\n    fn sides(&self) -> u32;\n    fn area(&self) -> u32 { self.sides() }\n}\nfn total() -> Node {\n    rsx! { div { onclick: move |_| { let v = json!(\"{x.area()}\"); } } }\n}\n";
const DISPATCHED: &str = "#[enum_dispatch]\ntrait Shape {\n    fn area(&self) -> u32;\n}\n";
const MARKER: &str = "trait Shape {}\n";
const CONSTANT_ONLY: &str = "trait Shape {\n    const SIDES: u32;\n}\n";
const SUPER: &str = "trait Shape {\n    fn area(&self) -> u32;\n}\ntrait Solid: Shape {}\n";
const PATHED: &str =
    "trait Shape {\n    fn area(&self) -> u32;\n}\nfn total<T: crate::Shape>(item: &T) -> u32 { item.area() }\n";
const TESTED: &str = "#[cfg(test)]\nmod tests {\n    trait Shape {\n        fn area(&self) -> u32;\n    }\n}\n";
const CONSUMER: &str = "fn total<T: Shape>(item: &T) -> u32 { item.area() }\n";
const TEST_CONSUMER: &str = "#[cfg(test)]\nmod tests {\n    fn total<T: Shape>(item: &T) -> u32 { item.area() }\n}\n";
const TWO: &str = "trait Shape {\n    fn area(&self) -> u32;\n}\nmod inner {\n    pub trait Solid {\n        fn volume(&self) -> u32;\n    }\n}\n";

const IN_MODULE: &str = "trait Shape {\n    fn area(&self) -> u32;\n}\nmod inner {\n    fn total<T: Shape>(item: &T) -> u32 { item.area() }\n}\n";

const CASES: [Case; 31] = [
    Case::new(
        Title::new("only impls"),
        &[Source::new(Located::new("a.rs"), Text::new(BARE))],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(Title::new("bound"), &[Source::new(Located::new("a.rs"), Text::new(BOUND))], Flagged::new(&[])),
    Case::new(Title::new("where clause"), &[Source::new(Located::new("a.rs"), Text::new(WHERE))], Flagged::new(&[])),
    Case::new(Title::new("dyn"), &[Source::new(Located::new("a.rs"), Text::new(DYN))], Flagged::new(&[])),
    Case::new(Title::new("impl trait"), &[Source::new(Located::new("a.rs"), Text::new(IMPL))], Flagged::new(&[])),
    Case::new(
        Title::new("provided method nobody calls"),
        &[Source::new(Located::new("a.rs"), Text::new(PROVIDED))],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(
        Title::new("provided method called as a method"),
        &[Source::new(Located::new("a.rs"), Text::new(PROVIDED_CALLED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("provided method called through a path"),
        &[Source::new(Located::new("a.rs"), Text::new(PROVIDED_PATH))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("provided method called only in tests"),
        &[Source::new(Located::new("a.rs"), Text::new(PROVIDED_ONLY_IN_TESTS))],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(
        Title::new("provided method called inside markup a macro does not parse as Rust"),
        &[Source::new(Located::new("a.rs"), Text::new(PROVIDED_IN_MARKUP))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("provided method called in nested markup"),
        &[Source::new(Located::new("a.rs"), Text::new(PROVIDED_NESTED_IN_MARKUP))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a field of the provided method's name in markup is no call"),
        &[Source::new(Located::new("a.rs"), Text::new(FIELD_IN_MARKUP))],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(
        Title::new("turbofish method call in markup"),
        &[Source::new(Located::new("a.rs"), Text::new(TURBOFISH_IN_MARKUP))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a free function after a range in markup is no method call"),
        &[Source::new(Located::new("a.rs"), Text::new(RANGE_IN_MARKUP))],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(
        Title::new("method call inside an interpolated markup string"),
        &[Source::new(Located::new("a.rs"), Text::new(INTERPOLATED_IN_MARKUP))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("path call inside markup"),
        &[Source::new(Located::new("a.rs"), Text::new(PATH_IN_MARKUP))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("string literal inside a non-formatting macro"),
        &[Source::new(Located::new("a.rs"), Text::new(LITERAL_IN_JSON))],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(
        Title::new("escaped braces are not placeholders"),
        &[Source::new(Located::new("a.rs"), Text::new(ESCAPED_BRACES))],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(
        Title::new("string literal inside an assertion macro"),
        &[Source::new(Located::new("a.rs"), Text::new(LITERAL_IN_ASSERT))],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(
        Title::new("string literal argument of a print macro"),
        &[Source::new(Located::new("a.rs"), Text::new(LITERAL_IN_PRINT))],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(
        Title::new("non-formatting macro nested in markup"),
        &[Source::new(Located::new("a.rs"), Text::new(JSON_IN_MARKUP))],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(
        Title::new("enum dispatch"),
        &[Source::new(Located::new("a.rs"), Text::new(DISPATCHED))],
        Flagged::new(&[]),
    ),
    Case::new(Title::new("marker"), &[Source::new(Located::new("a.rs"), Text::new(MARKER))], Flagged::new(&[])),
    Case::new(
        Title::new("constants only"),
        &[Source::new(Located::new("a.rs"), Text::new(CONSTANT_ONLY))],
        Flagged::new(&[]),
    ),
    Case::new(Title::new("supertrait"), &[Source::new(Located::new("a.rs"), Text::new(SUPER))], Flagged::new(&[])),
    Case::new(Title::new("pathed bound"), &[Source::new(Located::new("a.rs"), Text::new(PATHED))], Flagged::new(&[])),
    Case::new(Title::new("test module"), &[Source::new(Located::new("a.rs"), Text::new(TESTED))], Flagged::new(&[])),
    Case::new(
        Title::new("consumer in another file"),
        &[Source::new(Located::new("a.rs"), Text::new(BARE)), Source::new(Located::new("b.rs"), Text::new(CONSUMER))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("consumer only in tests"),
        &[
            Source::new(Located::new("a.rs"), Text::new(BARE)),
            Source::new(Located::new("b.rs"), Text::new(TEST_CONSUMER)),
        ],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(
        Title::new("nested module"),
        &[Source::new(Located::new("a.rs"), Text::new(TWO))],
        Flagged::new(&["a.rs:1", "a.rs:5"]),
    ),
    Case::new(
        Title::new("consumer in a module"),
        &[Source::new(Located::new("a.rs"), Text::new(IN_MODULE))],
        Flagged::new(&[]),
    ),
];

#[test]
fn traits_with_methods_and_no_consumer_are_flagged() {
    assert_eq!(Sources::mismatches::<UnconsumedTraits>(&CASES), Lines::default());
}

#[test]
fn a_finding_names_the_trait_and_both_fixes() {
    let text =
        UnconsumedTraits::report(&Sources::parsed(&[Source::new(Located::new("a.rs"), Text::new(BARE))])).to_string();
    assert!(text.contains("trait Shape has methods but nothing consumes it"), "{text}");
    assert!(text.contains("delete the trait and keep its methods inherent, or give it a consumer"), "{text}");
    assert_eq!(text.matches("helper").count(), 0);
}
