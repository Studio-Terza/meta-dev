use lint_fixtures::{Located, Source, Sources, Text};
use lint_transition_graph::DuplicateTransitions;

struct Duplicates;

impl Duplicates {
    fn of(sources: &[Source]) -> String {
        Sources::report::<DuplicateTransitions>(sources).to_string()
    }

    fn at(path: &'static str, text: &str) -> Source {
        Source::new(Located::new(path), Text::new(Box::leak(text.to_owned().into_boxed_str())))
    }

    fn shows(pair: &[&str]) -> bool {
        match pair {
            [text, expected] => Self::of(&[Self::at("crates/x/src/lib.rs", text)]).contains(expected),
            _ => false,
        }
    }
}

const TWO_PARSERS: &str = "struct Raw;\nstruct Policy;\nimpl From<&Raw> for Policy {\n    fn from(raw: &Raw) -> Self {}\n}\nimpl Raw {\n    fn policy(&self) -> Option<Policy> {}\n}\n";
const NO_CONVERSION: &str = "struct Raw;\nstruct Policy;\nimpl Raw {\n    fn policy(&self) -> Policy {}\n    fn parsed(&self) -> Result<Policy, Error> {}\n}\n";
const DIFFERENT_INPUTS: &str = "struct A;\nstruct B;\nstruct C;\nimpl A {\n    fn c(&self) -> C {}\n    fn c_with(&self, b: &B) -> C {}\n    fn b_and_a(b: B, a: A) -> C {}\n}\n";
const PRIMITIVE_PAIRS: &str = "struct A;\nimpl A {\n    fn ready(&self) -> bool {}\n    fn done(&self) -> bool {}\n    fn label(&self) -> PathBuf {}\n}\n";
const LIFTED: &str = "struct A;\nstruct B;\nimpl A {\n    fn b(&self) -> &B { &self.b }\n    fn made(&self) -> B {}\n}\nmod store {\n    impl A {\n        fn b_mut(&mut self) -> B {}\n    }\n}\n";

laws::table!(
    input_types_joined_to_one_output_more_than_once_are_listed,
    Duplicates::shows,
    [
        [TWO_PARSERS, "crates/x/src/lib.rs:4: x::Raw -> x::Policy has 2 transitions: x::Policy::from at crates/x/src/lib.rs:4, x::Raw::policy at crates/x/src/lib.rs:7; x::Policy::from is the standard conversion; keep the standard conversion and route the other transitions through it"] => true,
        [NO_CONVERSION, "x::Raw -> x::Policy has 2 transitions: x::Raw::policy at crates/x/src/lib.rs:4, x::Raw::parsed at crates/x/src/lib.rs:5; keep one transition, as From, TryFrom or FromStr where it fits"] => true,
        [DIFFERENT_INPUTS, "x::A, x::B -> x::C has 2 transitions: x::A::c_with at crates/x/src/lib.rs:6, x::A::b_and_a at crates/x/src/lib.rs:7"] => true,
        [DIFFERENT_INPUTS, "workspace: 3 written edges between domain types compared, 1 input-output pairs joined more than once by 2 edges; not compared: 0 touching a bare primitive, 0 touching an interface node or black box, 0 derived lifts and store operations"] => true,
        [PRIMITIVE_PAIRS, "has 2 transitions"] => false,
        [PRIMITIVE_PAIRS, "workspace: 0 written edges between domain types compared, 0 input-output pairs joined more than once by 0 edges; not compared: 2 touching a bare primitive, 1 touching an interface node or black box, 0 derived lifts and store operations"] => true,
        [LIFTED, "has 2 transitions"] => false,
        [LIFTED, "workspace: 1 written edges between domain types compared, 0 input-output pairs joined more than once by 0 edges; not compared: 0 touching a bare primitive, 0 touching an interface node or black box, 2 derived lifts and store operations"] => true,
        ["struct A;\n", "workspace:"] => false,
    ]
);

#[test]
fn duplicates_are_found_across_crates() {
    let text = Duplicates::of(&[
        Duplicates::at("crates/x/src/lib.rs", "struct Raw;\nimpl Raw {\n    fn policy(&self) -> Policy {}\n}\n"),
        Duplicates::at(
            "crates/y/src/lib.rs",
            "struct Policy;\nimpl Policy {\n    fn read(raw: &Raw) -> Policy {}\n}\n",
        ),
    ]);
    assert!(text.contains("x::Raw -> y::Policy has 2 transitions: x::Raw::policy at crates/x/src/lib.rs:3, y::Policy::read at crates/y/src/lib.rs:3"), "{text}");
}
