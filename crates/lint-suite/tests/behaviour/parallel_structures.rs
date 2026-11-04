use lint_core::Lint;
use lint_fixtures::{Located, Source, Sources, Text};
use lint_transition_graph::ParallelStructures;

use crate::shown::OnGraph;

struct Parallel;

impl Parallel {
    fn of(text: &&str) -> String {
        OnGraph::<ParallelStructures>::report(&[Sources::file("crates/x/src/lib.rs", Text::new(text))]).to_string()
    }

    fn structures(text: &&str) -> usize {
        Self::of(text).matches("alike under one renaming").count()
    }

    fn lawed(laws: &&'static str) -> usize {
        let sources = [
            Source::new(Located::new("crates/x/src/lib.rs"), Text::new(OPERATIONS)),
            Source::new(Located::new("crates/x/tests/t.rs"), Text::new(laws)),
        ];
        OnGraph::<ParallelStructures>::report(&Sources::parsed(&sources))
            .to_string()
            .matches("alike under one renaming")
            .count()
    }
}

const RENAMED: &str = "struct A;\nstruct B;\nstruct N;\nstruct M;\nimpl A {\n    fn name(self) -> String {}\n    fn next(self) -> N {}\n}\nimpl B {\n    fn label(self) -> String {}\n    fn after(self) -> M {}\n}\n";
const SHARED: &str = "struct A;\nstruct B;\nstruct C;\nimpl A {\n    fn name(self) -> String {}\n    fn next(self) -> C {}\n}\nimpl B {\n    fn label(self) -> String {}\n    fn after(self) -> C {}\n}\n";
const EXTRA: &str = "struct A;\nstruct B;\nstruct N;\nstruct M;\nimpl A {\n    fn name(self) -> String {}\n    fn next(self) -> N {}\n}\nimpl B {\n    fn label(self) -> String {}\n    fn after(self) -> M {}\n    fn count(self) -> usize {}\n}\n";
const NOT_ONE_TO_ONE: &str = "struct A;\nstruct B;\nstruct N;\nstruct O;\nstruct M;\nimpl A {\n    fn name(self) -> String {}\n    fn next(self) -> N {}\n    fn other(self) -> O {}\n}\nimpl B {\n    fn label(self) -> String {}\n    fn after(self) -> M {}\n}\n";
const ONE_MOVE: &str =
    "struct A;\nstruct B;\nimpl A {\n    fn name(self) -> String {}\n}\nimpl B {\n    fn label(self) -> String {}\n}\n";
const OPERATIONS: &str = "struct A;\nstruct B;\nimpl A {\n    fn join(self, other: A) -> A {}\n    fn name(self) -> String {}\n}\nimpl B {\n    fn merge(self, other: B) -> B {}\n    fn label(self) -> String {}\n}\n";
const SAME_LAWS: &str = "laws::associative!(a, any::<A>(), A::join);\nlaws::associative!(b, any::<B>(), B::merge);\n";
const OTHER_LAWS: &str = "laws::associative!(a, any::<A>(), A::join);\nlaws::commutative!(b, any::<B>(), B::merge);\n";
const ONE_SIDE: &str = "laws::associative!(a, any::<A>(), A::join);\n";
const THROUGH_A_TRAIT: &str = "struct A;\nstruct B;\nstruct N;\nstruct M;\ntrait Named {\n    fn name(&self) -> String;\n}\nimpl Named for A {\n    fn name(&self) -> String {}\n}\nimpl Named for B {\n    fn name(&self) -> String {}\n}\nimpl A {\n    fn next(self) -> N {}\n}\nimpl B {\n    fn after(self) -> M {}\n}\n";

laws::table!(
    types_are_parallel_when_their_moves_map_onto_each_other_under_one_renaming,
    Parallel::structures,
    [
        RENAMED => 1,
        SHARED => 1,
        EXTRA => 0,
        NOT_ONE_TO_ONE => 0,
        ONE_MOVE => 0,
        THROUGH_A_TRAIT => 1,
    ]
);

#[test]
fn a_structure_names_its_members_and_the_parameters_of_its_template() {
    let text = Parallel::of(&RENAMED);
    assert!(
        text.contains("crates/x/src/lib.rs:6: 2 types answer 2 moves alike under one renaming: x::A at crates/x/src/lib.rs:6, x::B at crates/x/src/lib.rs:10; one generic over 2 type parameters covers them; write the shared structure once"),
        "{text}"
    );
}

#[test]
fn a_type_both_members_reach_is_shared_and_not_a_parameter() {
    let text = Parallel::of(&SHARED);
    assert!(text.contains("one generic over 1 type parameters covers them"), "{text}");
}

#[test]
fn the_tally_counts_types_colours_and_structures() {
    let text = Parallel::of(&RENAMED);
    assert!(
        text.contains("workspace: 2 domain types with moves refine into 1 colours; 1 parallel structures hold 2 types"),
        "{text}"
    );
}

laws::table!(
    where_both_operations_declare_laws_the_laws_must_match,
    Parallel::lawed,
    [
        SAME_LAWS => 1,
        OTHER_LAWS => 0,
        ONE_SIDE => 1,
        "" => 1,
    ]
);
