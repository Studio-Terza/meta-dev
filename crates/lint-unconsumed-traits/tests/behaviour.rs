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

const CASES: [Case; 15] = [
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
        Title::new("provided method"),
        &[Source::new(Located::new("a.rs"), Text::new(PROVIDED))],
        Flagged::new(&[]),
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
];

#[test]
fn traits_with_methods_and_no_generic_consumer_are_flagged() {
    assert_eq!(Sources::mismatches::<UnconsumedTraits>(&CASES), Lines::default());
}

#[test]
fn a_finding_names_the_trait_and_the_fixes_in_order() {
    let text = Sources::report::<UnconsumedTraits>(&[Source::new(Located::new("a.rs"), Text::new(BARE))]).to_string();
    assert!(text.contains("trait Shape has no generic consumer"), "{text}");
    assert!(text.contains("a standard trait such as From, AsRef or Display, else write the generic over an existing trait, else a derive"), "{text}");
    assert!(!text.contains("helper"));
}
