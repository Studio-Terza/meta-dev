use lint_core::Lint;
use lint_fixtures::{Located, Source, Sources, Text};
use lint_transition_graph::TransitionGraph;

use crate::shown::{OnGraph, Shown};

struct Graphed;

impl Graphed {
    fn of(sources: &[Source]) -> String {
        OnGraph::<TransitionGraph>::report(&Sources::parsed(sources)).to_string()
    }

    fn at(path: &'static str, text: &str) -> Source {
        Source::new(Located::new(path), Text::new(Box::leak(text.to_owned().into_boxed_str())))
    }
}

const CONVERSIONS: &str = "struct Raw;\nstruct Policy;\nimpl From<Raw> for Policy {\n    fn from(raw: Raw) -> Self {}\n}\nimpl Policy {\n    fn parsed(raw: &Raw) -> Result<Policy, Error> {}\n}\n";
const PRIMITIVES: &str = "struct A;\nimpl A {\n    fn is_ready(&self) -> bool {}\n    fn label(&self) -> String {}\n    fn run(&mut self) {}\n    fn count(&self) -> Option<Vec<usize>> {}\n}\n";
const TRAIT_FIXED: &str = "struct A;\nimpl PartialEq for A {\n    fn eq(&self, other: &Self) -> bool {}\n}\nimpl fmt::Display for A {\n    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {}\n}\ntrait Local {\n    fn size(&self) -> usize;\n}\nimpl Local for A {\n    fn size(&self) -> usize {}\n}\n";
const GENERIC: &str = "struct A;\nstruct B;\nimpl A {\n    fn each<T: Clone>(&self, item: T) -> A {}\n    fn with(&self, f: impl Fn(&B) -> B) -> B {}\n    fn dynamic(&self) -> Box<dyn Render> {}\n    fn pair(&self) -> (A, B) {}\n    fn mixed(&self) -> (A, usize) {}\n}\n";
const LIFTS: &str = "struct A;\nstruct Name;\nimpl A {\n    fn name(&self) -> &Name { &self.name }\n    fn names(&self) -> Vec<Name> { self.items.iter().map(Name::from).collect() }\n    fn count(&self) -> usize { self.items.len() }\n}\nmod store {\n    impl Records {\n        fn insert(&mut self, name: Name) {}\n        fn find(&self, name: &Name) -> Option<A> {}\n    }\n}\n";
const ALGEBRA: &str = "struct A;\nstruct Raw;\nstruct Factor;\nimpl Add for A {\n    fn add(self, other: A) -> A {}\n}\nimpl A {\n    fn zero() -> A {}\n    fn scale(&self, factor: Factor) -> A {}\n    fn get(&self) -> Option<A> {}\n    fn bump(&mut self) {}\n    fn new(raw: Raw) -> A {}\n}\n";
const PORTED: &str = "struct A;\nstruct Raw;\nmod ports {\n    impl A {\n        fn read(raw: Raw) -> A {}\n        fn touch(&self) -> A {}\n    }\n}\n";
const LIFTED_ALGEBRA: &str = "struct A;\nimpl A {\n    fn all(&self) -> Vec<A> { self.items.iter().map(A::from).collect() }\n    fn same(&self) -> A {}\n}\n";
const OUTPUT: &str =
    "struct A;\nimpl Add for A {\n    type Output = A;\n    fn add(self, other: A) -> Self::Output {}\n}\n";
const FOREIGN: &str = "struct A;\nimpl Clone for A {\n    fn clone(&self) -> A {}\n}\nimpl Mul<f64> for A {\n    type Output = A;\n    fn mul(self, rhs: f64) -> A {}\n}\n";
const LOCAL_TRAIT: &str = "struct A;\ntrait Combine {\n    fn merge(self, other: A) -> A;\n}\nimpl Combine for A {\n    fn merge(self, other: A) -> A {}\n}\n";
const COLLECTED: &str =
    "struct A;\nimpl A {\n    fn children(&self) -> Vec<A> {}\n    fn others(&self) -> Option<Vec<A>> {}\n}\n";
const OUTPUT_MANY: &str =
    "struct A;\nimpl Add for A {\n    type Output = Vec<A>;\n    fn add(self, other: A) -> Self::Output {}\n}\n";
const OUTPUT_OPTIONAL: &str =
    "struct A;\nimpl Add for A {\n    type Output = Option<A>;\n    fn add(self, other: A) -> Self::Output {}\n}\n";
const INPUT_MANY: &str =
    "struct A;\nimpl A {\n    fn from_parts(parts: Vec<A>) -> A {}\n    fn from_maybe(part: Option<A>) -> A {}\n}\n";
const ARRAYED: &str = "struct A;\nimpl A {\n    fn pair(self) -> [A; 2] {}\n    fn sum(items: &[A]) -> A {}\n}\n";
const ACTED_WRAPPED: &str = "struct A;\nstruct F;\nimpl A {\n    fn with(self, f: Option<F>) -> A {}\n}\n";
const PARENTHESISED: &str = "struct A;\nimpl A {\n    fn get(&self) -> (Option<A>) {}\n}\n";
const TRIED: &str = "struct A;\nstruct E;\nimpl A {\n    fn try_bump(&mut self) -> Result<(), E> {}\n}\n";
const TESTED: &str = "struct A;\n#[cfg(test)]\nmod tests {\n    impl A {\n        fn f(&self) -> bool {}\n    }\n}\nimpl A {\n    #[cfg(test)]\n    fn g(&self) -> bool {}\n}\n";

laws::table!(
    the_transition_graph_of_a_crate_is_measured_with_its_coverage,
    Shown::of::<OnGraph<TransitionGraph>>,
    [
        [CONVERSIONS, "crates/x/src/lib.rs:4: crate x: 2 domain types, 0 interface nodes, 0 black boxes, 1 domain pairs, 2 written edges, coverage 2 of 2, share 1.000, 0 touch a bare primitive; of which 0 algebra operations, 0 partial; not written: 0 derived lifts, 0 store operations; give each bare primitive a newtype, an enum or a standard trait, and keep one transition per input-output pair"] => vec!["crates/x/src/lib.rs:4: crate x: 2 domain types, 0 interface nodes, 0 black boxes, 1 domain pairs, 2 written edges, coverage 2 of 2, share 1.000, 0 touch a bare primitive; of which 0 algebra operations, 0 partial; not written: 0 derived lifts, 0 store operations; give each bare primitive a newtype, an enum or a standard trait, and keep one transition per input-output pair"],
        [PRIMITIVES, "crate x: 1 domain types, 0 interface nodes, 1 black boxes, 0 domain pairs, 4 written edges, coverage 1 of 4, share 0.250, 3 touch a bare primitive; of which 1 algebra operations, 0 partial; not written: 0 derived lifts, 0 store operations"] => vec!["crate x: 1 domain types, 0 interface nodes, 1 black boxes, 0 domain pairs, 4 written edges, coverage 1 of 4, share 0.250, 3 touch a bare primitive; of which 1 algebra operations, 0 partial; not written: 0 derived lifts, 0 store operations"],
        [TRAIT_FIXED, "crate x: 1 domain types, 1 interface nodes, 2 black boxes, 0 domain pairs, 3 written edges, coverage 2 of 3, share 0.667, 1 touch a bare primitive"] => vec!["crate x: 1 domain types, 1 interface nodes, 2 black boxes, 0 domain pairs, 3 written edges, coverage 2 of 3, share 0.667, 1 touch a bare primitive"],
        [GENERIC, "crate x: 2 domain types, 3 interface nodes, 1 black boxes, 2 domain pairs, 5 written edges, coverage 4 of 5, share 0.800, 1 touch a bare primitive"] => vec!["crate x: 2 domain types, 3 interface nodes, 1 black boxes, 2 domain pairs, 5 written edges, coverage 4 of 5, share 0.800, 1 touch a bare primitive"],
        [LIFTS, "crate x: 2 domain types, 0 interface nodes, 2 black boxes, 2 domain pairs, 2 written edges, coverage 1 of 2, share 0.500, 1 touch a bare primitive; of which 0 algebra operations, 0 partial; not written: 2 derived lifts, 1 store operations"] => vec!["crate x: 2 domain types, 0 interface nodes, 2 black boxes, 2 domain pairs, 2 written edges, coverage 1 of 2, share 0.500, 1 touch a bare primitive; of which 0 algebra operations, 0 partial; not written: 2 derived lifts, 1 store operations"],
        [LIFTS, "crates/x/src/lib.rs:4: workspace: 2 written edges, coverage 1 of 2, share 0.500, 1 touch a bare primitive; of which 0 algebra operations, 0 partial; not written: 2 derived lifts, 1 store operations; give each bare primitive a newtype, an enum or a standard trait, so every written edge joins nodes the graph can see"] => vec!["crates/x/src/lib.rs:4: workspace: 2 written edges, coverage 1 of 2, share 0.500, 1 touch a bare primitive; of which 0 algebra operations, 0 partial; not written: 2 derived lifts, 1 store operations; give each bare primitive a newtype, an enum or a standard trait, so every written edge joins nodes the graph can see"],
        [ALGEBRA, "6 written edges, coverage 6 of 6, share 1.000, 0 touch a bare primitive; of which 5 algebra operations, 1 partial; not written: 0 derived lifts, 0 store operations"] => vec!["6 written edges, coverage 6 of 6, share 1.000, 0 touch a bare primitive; of which 5 algebra operations, 1 partial; not written: 0 derived lifts, 0 store operations"],
        [PORTED, "2 written edges, coverage 2 of 2, share 1.000, 0 touch a bare primitive; of which 1 algebra operations, 0 partial"] => vec!["2 written edges, coverage 2 of 2, share 1.000, 0 touch a bare primitive; of which 1 algebra operations, 0 partial"],
        [LIFTED_ALGEBRA, "1 written edges, coverage 1 of 1, share 1.000, 0 touch a bare primitive; of which 1 algebra operations, 0 partial; not written: 1 derived lifts, 0 store operations"] => vec!["1 written edges, coverage 1 of 1, share 1.000, 0 touch a bare primitive; of which 1 algebra operations, 0 partial; not written: 1 derived lifts, 0 store operations"],
        [OUTPUT, "1 written edges, coverage 1 of 1, share 1.000, 0 touch a bare primitive; of which 1 algebra operations, 0 partial"] => vec!["1 written edges, coverage 1 of 1, share 1.000, 0 touch a bare primitive; of which 1 algebra operations, 0 partial"],
        [FOREIGN, "2 written edges"] => vec!["2 written edges"],
        [FOREIGN, "of which 0 algebra operations, 0 partial"] => vec!["of which 0 algebra operations, 0 partial"],
        [LOCAL_TRAIT, "of which 1 algebra operations, 0 partial"] => vec!["of which 1 algebra operations, 0 partial"],
        [COLLECTED, "2 written edges"] => vec!["2 written edges"],
        [COLLECTED, "of which 0 algebra operations, 0 partial"] => vec!["of which 0 algebra operations, 0 partial"],
        [OUTPUT_MANY, "of which 0 algebra operations, 0 partial"] => vec!["of which 0 algebra operations, 0 partial"],
        [OUTPUT_OPTIONAL, "of which 1 algebra operations, 1 partial"] => vec!["of which 1 algebra operations, 1 partial"],
        [INPUT_MANY, "of which 0 algebra operations, 0 partial"] => vec!["of which 0 algebra operations, 0 partial"],
        [ARRAYED, "of which 0 algebra operations, 0 partial"] => vec!["of which 0 algebra operations, 0 partial"],
        [ACTED_WRAPPED, "of which 1 algebra operations, 0 partial"] => vec!["of which 1 algebra operations, 0 partial"],
        [PARENTHESISED, "of which 1 algebra operations, 1 partial"] => vec!["of which 1 algebra operations, 1 partial"],
        [TRIED, "of which 0 algebra operations, 0 partial"] => vec!["of which 0 algebra operations, 0 partial"],
        [TESTED, "crate x"] => Vec::<String>::new(),
        [TESTED, "workspace:"] => Vec::<String>::new(),
        ["struct A;\n", "crate x"] => Vec::<String>::new(),
    ]
);

#[test]
fn a_type_resolves_to_its_own_crate_first_and_else_to_the_one_crate_declaring_it() {
    let text = Graphed::of(&[
        Graphed::at(
            "crates/x/src/lib.rs",
            "struct A;\nimpl A {\n    fn b(&self) -> B {}\n    fn c(&self) -> C {}\n}\n",
        ),
        Graphed::at("crates/y/src/lib.rs", "struct B;\nstruct C;\n"),
        Graphed::at(
            "crates/z/src/lib.rs",
            "struct C;\nstruct A;\nimpl A {\n    fn b(&self) -> B {}\n    fn is(&self) -> bool {}\n}\n",
        ),
    ]);
    assert!(text.contains("crate x: 1 domain types, 0 interface nodes, 0 black boxes, 1 domain pairs, 2 written edges, coverage 2 of 2, share 1.000"), "{text}");
    assert!(text.contains("crate z: 2 domain types, 0 interface nodes, 0 black boxes, 1 domain pairs, 2 written edges, coverage 1 of 2, share 0.500"), "{text}");
    assert!(
        text.contains(
            "crates/x/src/lib.rs:3: workspace: 4 written edges, coverage 3 of 4, share 0.750, 1 touch a bare primitive"
        ),
        "{text}"
    );
}
