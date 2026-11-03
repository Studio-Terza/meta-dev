use lint_fixtures::{Located, Source, Sources, Text};
use lint_transition_graph::{DuplicateTransitions, OneEdge};

struct Duplicates;

impl Duplicates {
    fn of(sources: &[Source]) -> String {
        format!("{}{}", Sources::report::<OneEdge>(sources), Sources::report::<DuplicateTransitions>(sources))
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

#[test]
fn a_name_declared_in_several_crates_resolves_through_its_import() {
    let text = Duplicates::of(&[
        Duplicates::at("crates/x/src/lib.rs", "struct Report;\n"),
        Duplicates::at("crates/y/src/lib.rs", "struct Report;\nstruct File;\n"),
        Duplicates::at(
            "crates/z/src/lib.rs",
            "use y::{File, Report};\nstruct One;\nimpl One {\n    fn report(file: &File) -> Report {}\n}\nstruct Two;\nimpl Two {\n    fn report(file: &File) -> y::Report {}\n}\n",
        ),
    ]);
    assert!(text.contains("y::File -> y::Report has 2 transitions"), "{text}");
}

#[test]
fn a_foreign_import_is_a_black_box_even_when_the_workspace_declares_the_name() {
    let text = Duplicates::of(&[
        Duplicates::at("crates/x/src/lib.rs", "struct Command;\nstruct Line;\n"),
        Duplicates::at(
            "crates/y/src/lib.rs",
            "use std::process::Command;\nuse x::Line;\nstruct Run;\nimpl Run {\n    fn first(line: &Line) -> Command {}\n    fn second(line: &Line) -> Command {}\n}\n",
        ),
    ]);
    assert!(!text.contains("has 2 transitions"), "{text}");
}

#[test]
fn an_import_from_a_child_module_stays_in_the_crate() {
    let text = Duplicates::of(&[
        Duplicates::at("crates/x/src/lib.rs", "struct Policy;\n"),
        Duplicates::at(
            "crates/y/src/lib.rs",
            "mod inner;\nuse inner::Policy;\nstruct Raw;\nimpl Raw {\n    fn policy(&self) -> Policy {}\n    fn read(&self) -> Policy {}\n}\n",
        ),
        Duplicates::at("crates/y/src/inner.rs", "pub struct Policy;\n"),
    ]);
    assert!(text.contains("y::Raw -> y::Policy has 2 transitions"), "{text}");
}

#[test]
fn impls_of_one_trait_on_different_types_are_different_edges() {
    let text = Duplicates::of(&[Duplicates::at(
        "crates/x/src/lib.rs",
        "struct File;\nstruct Report;\nstruct One;\nstruct Two;\nimpl Check for One {\n    fn report(file: &File) -> Report {}\n}\nimpl Check for Two {\n    fn report(file: &File) -> Report {}\n}\n",
    )]);
    assert!(!text.contains("has 2 transitions"), "{text}");
}

#[test]
fn an_inherent_function_beside_a_trait_impl_on_another_type_is_a_duplicate() {
    let text = Duplicates::of(&[Duplicates::at(
        "crates/x/src/lib.rs",
        "struct File;\nstruct Report;\nstruct One;\nstruct Two;\nimpl Check for One {\n    fn report(file: &File) -> Report {}\n}\nimpl Two {\n    fn report(file: &File) -> Report {}\n    fn again(file: &File) -> Report {}\n}\n",
    )]);
    assert!(text.contains("x::File -> x::Report has 2 transitions: x::Two::report"), "{text}");
}

#[test]
fn a_name_declared_twice_in_one_crate_is_a_leaf() {
    let text = Duplicates::of(&[
        Duplicates::at("crates/x/src/lib.rs", "mod one;\nmod two;\nstruct Fix;\n"),
        Duplicates::at("crates/x/src/one.rs", "struct Tally;\nimpl Tally {\n    fn fix(&self) -> Fix {}\n}\n"),
        Duplicates::at("crates/x/src/two.rs", "struct Tally;\nimpl Tally {\n    fn fix(&self) -> Fix {}\n}\n"),
    ]);
    assert!(!text.contains("has 2 transitions"), "{text}");
}
