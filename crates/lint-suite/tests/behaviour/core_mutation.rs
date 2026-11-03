use lint_transition_graph::CoreMutation;

use crate::shown::Shown;

const MUTATION: &str = "struct A;\nimpl A {\n    fn run(&mut self) {}\n    fn fill(&self, out: &mut Vec<A>) {}\n    fn both(&mut self, out: &mut A) {}\n    fn pure(&self) -> A {}\n}\nimpl fmt::Display for A {\n    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {}\n}\nimpl Iterator for A {\n    fn next(&mut self) -> Option<A> {}\n}\nimpl AsMut<A> for B {\n    fn as_mut(&mut self) -> &mut A { &mut self.a }\n}\ntrait Local {\n    fn bump(&mut self);\n}\nimpl Local for A {\n    fn bump(&mut self) {}\n}\nmod ports {\n    fn write(out: &mut A) {}\n}\nmod boundary {\n    fn scan(out: &mut A) {}\n}\nmod store {\n    impl Records {\n        fn insert(&mut self, record: A, f: impl Fn(u8)) -> Option<usize> {}\n        fn merge(&self, into: &mut A) {}\n    }\n}\n";
const PURE: &str = "struct A;\nimpl A {\n    fn pure(&self) -> A {}\n}\nmod ports {\n    fn write(out: &mut A) {}\n}\n";

laws::table!(
    core_edges_taking_mut_are_counted_apart_from_ports_boundaries_stores_and_foreign_trait_signatures,
    Shown::of::<CoreMutation>,
    [
        [MUTATION, "crates/x/src/lib.rs:3: crate x: 7 core edges take &mut, 5 through &mut self, 2 through a &mut parameter only, 3 with a signature fixed by a foreign trait, 4 to migrate; 1 store operations, x::Records::insert applies x::A, impl Fn -> usize; move each mutation into a store operation, a &mut self method in a store module, or return a new value"] => true,
        [MUTATION, "crates/x/src/lib.rs:3: workspace: 7 core edges take &mut, 5 through &mut self, 2 through a &mut parameter only, 3 with a signature fixed by a foreign trait, 4 to migrate; 1 store operations; move each mutation"] => true,
        [PURE, "crate x"] => false,
        [PURE, "workspace:"] => false,
    ]
);
