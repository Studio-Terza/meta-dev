use lint_core::Lint;
use lint_fixtures::{Located, Source, Sources, Text};
use lint_transition_graph::Homomorphisms;

use crate::shown::OnGraph;

struct Mapped;

impl Mapped {
    fn of(sources: &[Source]) -> String {
        OnGraph::<Homomorphisms>::report(&Sources::parsed(sources)).to_string()
    }

    fn lawed(source: &'static str, law: &'static str) -> String {
        Self::of(&[Self::at("crates/x/src/lib.rs", source), Self::at("crates/x/tests/t.rs", law)])
    }

    fn at(path: &'static str, text: &'static str) -> Source {
        Source::new(Located::new(path), Text::new(text))
    }
}

const EDGE: &str = "struct A;\nstruct B;\nimpl A {\n    fn join(self, other: A) -> A {}\n    fn into_b(&self) -> B {}\n}\nimpl B {\n    fn join(self, other: B) -> B {}\n}\n";
const SECOND: &str = "struct A;\nstruct B;\nimpl A {\n    fn join(self, other: A) -> A {}\n    fn into_b(&self) -> B {}\n    fn other_b(&self) -> B {}\n}\nimpl B {\n    fn join(self, other: B) -> B {}\n}\n";
const OPLESS_TARGET: &str =
    "struct A;\nstruct B;\nimpl A {\n    fn join(self, other: A) -> A {}\n    fn into_b(&self) -> B {}\n}\n";
const OPLESS_SOURCE: &str = "struct A;\nstruct B;\nimpl A {\n    fn into_b(&self) -> B {}\n}\nimpl B {\n    fn join(self, other: B) -> B {}\n}\n";
const PRESERVING: &str = "laws::preserves!(into_b_keeps_join, any::<A>(), A::join, B::join, A::into_b);\n";
const PRESERVING_OTHER: &str = "laws::preserves!(other_b_keeps_join, any::<A>(), A::join, B::join, A::other_b);\n";
const BARE: &str = "preserves!(into_b_keeps_join, any::<A>(), A::join, B::join, A::into_b);\n";
const NESTED: &str =
    "#[cfg(test)]\nmod tests {\n    laws::preserves!(keeps, any::<A>(), A::join, B::join, A::into_b);\n}\n";
const IN_FILE: &str = "struct A;\nstruct B;\nimpl A {\n    fn join(self, other: A) -> A {}\n    fn into_b(&self) -> B {}\n}\nimpl B {\n    fn join(self, other: B) -> B {}\n}\n#[cfg(test)]\nmod tests {\n    laws::preserves!(keeps, any::<A>(), A::join, B::join, A::into_b);\n}\n";
const SOURCE_UNNAMED: &str = "laws::preserves!(keeps, any::<u8>(), join, B::join, into_b);\n";
const TARGET_UNNAMED: &str = "laws::preserves!(keeps, any::<u8>(), A::join, join, into_b);\n";
const EDGE_UNNAMED: &str = "laws::preserves!(keeps, any::<A>(), A::join, B::join, edge);\n";
const TITLED: &str = "laws::preserves!(A_B_into_b, any::<u8>(), join, join, other);\n";
const ASSOCIATIVE: &str = "laws::associative!(keeps, any::<A>(), A::join);\nlaws::associative!(keeps_b, any::<B>(), B::join);\nlaws::commutative!(edge, any::<A>(), A::into_b);\n";
const PLAIN_LAW: &str = "laws::law!(keeps, any::<A>(), value, { A::join(B::join(A::into_b(value))) });\n";
const FOREIGN_MACRO: &str = "other::preserves!(keeps, any::<A>(), A::join, B::join, A::into_b);\n";
const TWO_INPUTS: &str = "struct A;\nstruct B;\nstruct C;\nimpl A {\n    fn join(self, other: A) -> A {}\n    fn combine(&self, other: &C) -> B {}\n}\nimpl B {\n    fn join(self, other: B) -> B {}\n}\nimpl C {\n    fn join(self, other: C) -> C {}\n}\n";
const SAME_INPUT_TWICE: &str = "struct A;\nstruct B;\nimpl A {\n    fn join(self, other: A) -> A {}\n    fn pair(&self, other: &A) -> B {}\n}\nimpl B {\n    fn join(self, other: B) -> B {}\n}\n";
const PRIMITIVE_INPUT: &str = "struct A;
struct B;
impl A {
    fn join(self, other: A) -> A {}
}
impl B {
    fn join(self, other: B) -> B {}
    fn from_count(count: u8) -> B {}
}
impl A {
    fn scaled(&self, count: u8) -> B {}
}
";
const PRIMITIVE_OUTPUT: &str =
    "struct A;\nimpl A {\n    fn join(self, other: A) -> A {}\n    fn count(&self) -> u8 {}\n}\n";
const SAME_TYPE: &str = "struct A;\nimpl A {\n    fn join(self, other: A) -> A {}\n    fn all(&self) -> Vec<A> {}\n}\n";
const LIFTED: &str = "struct A;\nstruct B;\nimpl A {\n    fn join(self, other: A) -> A {}\n    fn b(&self) -> &B { &self.b }\n}\nimpl B {\n    fn join(self, other: B) -> B {}\n}\n";
const PORTED: &str = "struct A;\nstruct B;\nimpl A {\n    fn join(self, other: A) -> A {}\n}\nimpl B {\n    fn join(self, other: B) -> B {}\n}\nmod ports {\n    impl A {\n        fn read(&self) -> B {}\n    }\n}\n";
const PARTIAL_EDGE: &str = "struct A;\nstruct B;\nimpl A {\n    fn join(self, other: A) -> A {}\n    fn try_b(&self) -> Option<B> {}\n}\nimpl B {\n    fn join(self, other: B) -> B {}\n}\n";
const OPERATION_ONLY: &str = "struct A;\nimpl A {\n    fn join(self, other: A) -> A {}\n}\n";
const ACROSS_CRATES: &str = "struct A;\nstruct B;\nimpl A {\n    fn join(self, other: A) -> A {}\n    fn into_b(&self) -> B {}\n}\nimpl B {\n    fn join(self, other: B) -> B {}\n}\n";
const BACK: &str = "struct A;\nstruct B;\nimpl A {\n    fn join(self, other: A) -> A {}\n    fn into_b(&self) -> B {}\n}\nimpl B {\n    fn join(self, other: B) -> B {}\n    fn into_a(&self) -> A {}\n}\n";

#[test]
fn an_edge_between_algebraic_types_is_listed_with_its_site_and_the_fix() {
    let text = Mapped::of(&[Mapped::at("crates/x/src/lib.rs", EDGE)]);
    assert!(
        text.contains("crates/x/src/lib.rs:5: crate x: 1 edges between algebraic types, 0 declare preservation, 1 do not; x::A -> x::B: into_b at crates/x/src/lib.rs:5; declare the edge with laws::preserves! in a test of the same package, calling an operation of each type and the edge function by path (Type::function) or in a closure"),
        "{text}"
    );
}

#[test]
fn a_preserves_law_naming_both_types_and_the_function_clears_the_edge() {
    assert!(
        [PRESERVING, NESTED, BARE]
            .iter()
            .all(|law| Mapped::lawed(EDGE, law).matches("edges between algebraic types").count() == 0)
    );
}

#[test]
fn a_preserves_law_in_a_test_module_of_the_file_clears_the_edge() {
    let text = Mapped::of(&[Mapped::at("crates/x/src/lib.rs", IN_FILE)]);
    assert_eq!(text.matches("edges between algebraic types").count(), 0, "{text}");
}

#[test]
fn a_law_naming_only_part_of_the_edge_does_not_clear_it() {
    assert!(
        [SOURCE_UNNAMED, TARGET_UNNAMED, EDGE_UNNAMED, TITLED, ASSOCIATIVE, PLAIN_LAW, FOREIGN_MACRO]
            .iter()
            .all(|law| Mapped::lawed(EDGE, law).contains("1 do not; x::A -> x::B: into_b"))
    );
}

#[test]
fn a_law_of_another_package_does_not_clear_the_edge() {
    let text = Mapped::of(&[Mapped::at("crates/x/src/lib.rs", EDGE), Mapped::at("crates/y/tests/t.rs", PRESERVING)]);
    assert!(text.contains("crate x: 1 edges between algebraic types, 0 declare preservation, 1 do not"), "{text}");
}

#[test]
fn counts_are_per_crate_and_declared_edges_are_counted() {
    let text = Mapped::of(&[
        Mapped::at("crates/x/src/lib.rs", SECOND),
        Mapped::at("crates/x/tests/t.rs", PRESERVING),
        Mapped::at("crates/y/src/lib.rs", ACROSS_CRATES),
    ]);
    assert!(text.contains("crate x: 2 edges between algebraic types, 1 declare preservation, 1 do not; x::A -> x::B: other_b at crates/x/src/lib.rs:6;"), "{text}");
    assert!(text.contains("crate y: 1 edges between algebraic types, 0 declare preservation, 1 do not"), "{text}");
}

#[test]
fn every_undeclared_edge_is_listed_in_order() {
    let text = Mapped::lawed(SECOND, PRESERVING_OTHER);
    assert!(
        text.contains("1 declare preservation, 1 do not; x::A -> x::B: into_b at crates/x/src/lib.rs:5;"),
        "{text}"
    );
    let both = Mapped::of(&[Mapped::at("crates/x/src/lib.rs", SECOND)]);
    assert!(
        both.contains(
            "2 do not; x::A -> x::B: into_b at crates/x/src/lib.rs:5; x::A -> x::B: other_b at crates/x/src/lib.rs:6;"
        ),
        "{both}"
    );
    assert!(both.contains("crates/x/src/lib.rs:5: crate x"), "{both}");
}

#[test]
fn edges_in_both_directions_are_listed() {
    let text = Mapped::of(&[Mapped::at("crates/x/src/lib.rs", BACK)]);
    assert!(text.contains("2 edges between algebraic types, 0 declare preservation, 2 do not"), "{text}");
    assert!(text.contains("x::A -> x::B: into_b"), "{text}");
    assert!(text.contains("x::B -> x::A: into_a"), "{text}");
}

#[test]
fn an_edge_to_a_type_with_no_operations_is_out_of_scope() {
    assert!(
        [OPLESS_TARGET, OPLESS_SOURCE, OPERATION_ONLY].iter().all(|source| Mapped::of(&[Mapped::at(
            "crates/x/src/lib.rs",
            source
        )])
        .matches("crate x")
        .count()
            == 0)
    );
}

#[test]
fn edges_that_are_not_one_domain_input_to_a_different_domain_output_are_out_of_scope() {
    assert!(
        [TWO_INPUTS, SAME_INPUT_TWICE, PRIMITIVE_OUTPUT, SAME_TYPE, LIFTED, PORTED].iter().all(|source| Mapped::of(&[
            Mapped::at("crates/x/src/lib.rs", source)
        ])
        .matches("crate x")
        .count()
            == 0)
    );
}

#[test]
fn a_primitive_beside_the_one_domain_input_does_not_change_the_scope() {
    let text = Mapped::of(&[Mapped::at("crates/x/src/lib.rs", PRIMITIVE_INPUT)]);
    assert!(
        text.contains("1 edges between algebraic types, 0 declare preservation, 1 do not; x::A -> x::B: scaled"),
        "{text}"
    );
}

#[test]
fn a_partial_edge_is_listed() {
    let text = Mapped::of(&[Mapped::at("crates/x/src/lib.rs", PARTIAL_EDGE)]);
    assert!(text.contains("x::A -> x::B: try_b"), "{text}");
}

const UNRELATED: &str = "laws::preserves!(keeps, any::<A>(), A::zzz, B::yyy, A::into_b);\n";
const SOURCE_ONLY: &str = "laws::preserves!(keeps, any::<A>(), A::join, B::yyy, A::into_b);\n";
const TARGET_ONLY: &str = "laws::preserves!(keeps, any::<A>(), A::zzz, B::join, A::into_b);\n";
const OTHER_OPS: &str = "struct A;\nstruct B;\nimpl A {\n    fn join(self, other: A) -> A {}\n    fn meet(self, other: A) -> A {}\n    fn into_b(&self) -> B {}\n}\nimpl B {\n    fn join(self, other: B) -> B {}\n    fn meet(self, other: B) -> B {}\n}\n";
const MEETING: &str = "laws::preserves!(keeps, any::<A>(), A::meet, B::meet, A::into_b);\n";

#[test]
fn a_preserves_law_naming_unrelated_operations_does_not_clear_the_edge() {
    assert!(
        [UNRELATED, SOURCE_ONLY, TARGET_ONLY]
            .iter()
            .all(|law| Mapped::lawed(EDGE, law).contains("1 do not; x::A -> x::B: into_b"))
    );
}

#[test]
fn a_preserves_law_naming_any_operation_of_each_type_clears_the_edge() {
    assert_eq!(Mapped::lawed(OTHER_OPS, MEETING).matches("edges between algebraic types").count(), 0);
}

const FROM_IMPL: &str = "struct A;\nstruct B;\nimpl A {\n    fn join(self, other: A) -> A {}\n}\nimpl B {\n    fn join(self, other: B) -> B {}\n}\nimpl From<A> for B {\n    fn from(a: A) -> B {}\n}\n";
const CONSTRUCTOR: &str = "struct A;\nstruct B;\nimpl A {\n    fn join(self, other: A) -> A {}\n}\nimpl B {\n    fn join(self, other: B) -> B {}\n    fn of(a: A) -> B {}\n}\n";

#[test]
fn a_preserves_law_clears_an_edge_whose_function_belongs_to_the_output_type() {
    let cleared: Vec<String> = [
        (FROM_IMPL, "laws::preserves!(keeps, any::<A>(), A::join, B::join, B::from);\n"),
        (FROM_IMPL, "laws::preserves!(keeps, any::<A>(), A::join, B::join, <B as From<A>>::from);\n"),
        (FROM_IMPL, "laws::preserves!(keeps, any::<A>(), A::join, B::join, |a: A| B::from(a));\n"),
        (CONSTRUCTOR, "laws::preserves!(keeps, any::<A>(), A::join, B::join, B::of);\n"),
    ]
    .iter()
    .map(|&(source, law)| Mapped::lawed(source, law))
    .filter(|text| text.contains("edges between algebraic types"))
    .collect();
    assert!(cleared.is_empty(), "{cleared:?}");
}

const ADDED: &str = "struct A;\nstruct B;\nimpl Add for A {\n    type Output = A;\n    fn add(self, other: A) -> A {}\n}\nimpl Add for B {\n    type Output = B;\n    fn add(self, other: B) -> B {}\n}\nimpl A {\n    fn into_b(&self) -> B {}\n}\n";

#[test]
fn an_output_side_trait_path_or_untyped_closure_resolves_to_the_output_type() {
    let listed: Vec<String> = [
        "laws::preserves!(keeps, any::<A>(), A::add, Add::add, A::into_b);\n",
        "laws::preserves!(keeps, any::<A>(), A::add, |b| Add::add(b, b), A::into_b);\n",
        "laws::preserves!(keeps, any::<A>(), Add::add, |a, b| a.add(b), A::into_b);\n",
    ]
    .iter()
    .map(|law| Mapped::lawed(ADDED, law))
    .filter(|text| text.contains("edges between algebraic types"))
    .collect();
    assert!(listed.is_empty(), "{listed:?}");
}

#[test]
fn a_preserves_law_that_misses_a_side_says_which_side_and_why() {
    let text = Mapped::lawed(EDGE, "laws::preserves!(keeps, any::<A>(), A::join, A::join, A::into_b);\n");
    assert!(
        text.contains("x::A -> x::B: into_b at crates/x/src/lib.rs:5 (law preserves! at crates/x/tests/t.rs:1: output side resolves to A::join, not an operation of x::B)"),
        "{text}"
    );
}

#[test]
fn an_unreadable_closure_in_a_preserves_law_is_reported_on_its_own() {
    let text = Mapped::lawed(
        OPERATION_ONLY,
        "laws::preserves!(keeps, any::<A>(), A::join, |a: B, b: B| combine(a, b), A::into_b);\n",
    );
    assert!(
        text.contains("crates/x/tests/t.rs:1: law preserves! checks a closure the lint cannot resolve to an operation; pass the operation path"),
        "{text}"
    );
}
