use lint_core::Lint;
use lint_fixtures::{Located, Source, Sources, Text};
use lint_transition_graph::ParallelStructures;

use crate::shown::OnGraph;
use lint_fixtures::Shown;

struct Parallel;

impl Parallel {
    fn of(text: &&str) -> String {
        OnGraph::<ParallelStructures>::report(&[Sources::file("crates/x/src/lib.rs", Text::new(text))]).to_string()
    }

    fn structures(text: &&str) -> usize {
        Self::of(text).matches("alike under one renaming").count()
    }

    fn lawed(laws: &&'static str) -> usize {
        Self::with_laws(OPERATIONS, laws).matches("alike under one renaming").count()
    }

    fn with_laws(source: &'static str, laws: &'static str) -> String {
        let sources = [
            Source::new(Located::new("crates/x/src/lib.rs"), Text::new(source)),
            Source::new(Located::new("crates/x/tests/t.rs"), Text::new(laws)),
        ];
        OnGraph::<ParallelStructures>::report(&Sources::parsed(&sources)).to_string()
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
const SAME_SHAPE_TWICE: &str = "struct A;\nstruct B;\nstruct N;\nstruct M;\nimpl A {\n    fn name(self) -> String {}\n    fn next(self) -> N {}\n}\nimpl B {\n    fn label(self) -> String {}\n    fn title(self) -> String {}\n    fn after(self) -> M {}\n}\n";
const PORTED: &str = "struct A;\nstruct B;\nstruct N;\nstruct M;\nmod ports {\n    impl A {\n        fn read(self) -> String {}\n    }\n}\nimpl A {\n    fn next(self) -> N {}\n}\nimpl B {\n    fn label(self) -> String {}\n    fn after(self) -> M {}\n}\n";
const WITH_INPUT: &str = "struct A;\nstruct B;\nstruct N;\nimpl A {\n    fn name(self, n: N) -> String {}\n    fn label(self) -> String {}\n}\nimpl B {\n    fn name(self) -> String {}\n    fn label(self) -> String {}\n}\n";
const THREE_WAYS: &str = "struct A;\nstruct B;\nstruct C;\nstruct K;\nstruct L;\nstruct M;\nstruct N;\nimpl A {\n    fn next(self) -> N {}\n    fn prev(self) -> K {}\n}\nimpl B {\n    fn next(self) -> M {}\n    fn prev(self) -> K {}\n}\nimpl C {\n    fn next(self) -> N {}\n    fn prev(self) -> L {}\n}\n";
const TWO_OPERATIONS: &str = "struct A;\nstruct B;\nimpl A {\n    fn join(self, other: A) -> A {}\n    fn meet(self, other: A) -> A {}\n    fn name(self) -> String {}\n}\nimpl B {\n    fn merge(self, other: B) -> B {}\n    fn unite(self, other: B) -> B {}\n    fn label(self) -> String {}\n}\n";
const BOTH_MATCH: &str = "laws::associative!(a, any::<A>(), A::join);\nlaws::commutative!(b, any::<A>(), A::meet);\nlaws::associative!(c, any::<B>(), B::merge);\nlaws::commutative!(d, any::<B>(), B::unite);\n";
const SECOND_DIFFERS: &str = "laws::associative!(a, any::<A>(), A::join);\nlaws::commutative!(b, any::<A>(), A::meet);\nlaws::associative!(c, any::<B>(), B::merge);\nlaws::associative!(d, any::<B>(), B::unite);\n";
const THREE_OPERATIONS: &str = "struct A;\nstruct B;\nstruct C;\nimpl A {\n    fn join(self, other: A) -> A {}\n    fn name(self) -> String {}\n}\nimpl B {\n    fn merge(self, other: B) -> B {}\n    fn label(self) -> String {}\n}\nimpl C {\n    fn unite(self, other: C) -> C {}\n    fn title(self) -> String {}\n}\n";
const FIRST_DIFFERS: &str = "laws::associative!(a, any::<A>(), A::join);\nlaws::commutative!(b, any::<B>(), B::merge);\nlaws::commutative!(c, any::<C>(), C::unite);\n";
const SAME_IMPL_BODIES: &str = "struct A;\nstruct B;\nstruct N;\nstruct M;\ntrait Show {\n    fn show(&self) -> String;\n}\nimpl A {\n    fn name(self) -> String {}\n    fn next(self) -> N {}\n}\nimpl B {\n    fn label(self) -> String {}\n    fn after(self) -> M {}\n}\nimpl Show for A {\n    fn show(&self) -> String {\n        self.name()\n    }\n}\nimpl Show for B {\n    fn show(&self) -> String {\n        self.label()\n    }\n}\n";
const OTHER_IMPL_BODIES: &str = "struct A;\nstruct B;\nstruct N;\nstruct M;\ntrait Show {\n    fn show(&self) -> String;\n}\nimpl A {\n    fn name(self) -> String {}\n    fn next(self) -> N {}\n}\nimpl B {\n    fn label(self) -> String {}\n    fn after(self) -> M {}\n}\nimpl Show for A {\n    fn show(&self) -> String {\n        self.name()\n    }\n}\nimpl Show for B {\n    fn show(&self) -> String {\n        format!(\"{}\", 1)\n    }\n}\n";
const ONE_SIDE_FILLS: &str = "struct A;\nstruct B;\nstruct N;\nstruct M;\ntrait Show {\n    fn show(&self) -> String;\n}\nimpl A {\n    fn name(self) -> String {}\n    fn next(self) -> N {}\n}\nimpl B {\n    fn label(self) -> String {}\n    fn after(self) -> M {}\n}\nimpl Show for A {\n    fn show(&self) -> String {\n        self.name()\n    }\n}\n";
const OTHER_IMPL_BODIES_AND_A_THIRD_FILLER: &str = "struct A;\nstruct B;\nstruct N;\nstruct M;\ntrait Show {\n    fn show(&self) -> String;\n}\nimpl A {\n    fn name(self) -> String {}\n    fn next(self) -> N {}\n}\nimpl B {\n    fn label(self) -> String {}\n    fn after(self) -> M {}\n}\nimpl Show for A {\n    fn show(&self) -> String {\n        self.name()\n    }\n}\nimpl Show for B {\n    fn show(&self) -> String {\n        format!(\"{}\", 1)\n    }\n}\nstruct Z;\nimpl Show for Z {\n    fn show(&self) -> String {\n        String::new()\n    }\n}\n";
const OTHER_IMPL_BODY_IN_A_MODULE: &str = "struct A;\nstruct B;\nstruct N;\nstruct M;\ntrait Show {\n    fn show(&self) -> String;\n}\nimpl A {\n    fn name(self) -> String {}\n    fn next(self) -> N {}\n}\nimpl B {\n    fn label(self) -> String {}\n    fn after(self) -> M {}\n}\nimpl Show for A {\n    fn show(&self) -> String {\n        self.name()\n    }\n}\nmod inner {\n    impl Show for super::B {\n        fn show(&self) -> String {\n            format!(\"{}\", 1)\n        }\n    }\n}\n";
const RENAMED_INPUTS: &str = "struct A;\nstruct B;\nstruct N;\nstruct M;\nimpl A {\n    fn name(self) -> String {}\n    fn with(self, n: N) -> String {}\n}\nimpl B {\n    fn label(self) -> String {}\n    fn with(self, m: M) -> String {}\n}\n";
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
        THROUGH_A_TRAIT => 0,
        SAME_IMPL_BODIES => 1,
        OTHER_IMPL_BODIES => 0,
        OTHER_IMPL_BODIES_AND_A_THIRD_FILLER => 0,
        OTHER_IMPL_BODY_IN_A_MODULE => 0,
        ONE_SIDE_FILLS => 1,
        SAME_SHAPE_TWICE => 0,
        PORTED => 0,
        WITH_INPUT => 0,
    ]
);

#[test]
fn a_structure_names_its_members_and_the_parameters_of_its_template() {
    let text = Parallel::of(&RENAMED);
    assert!(
        text.contains("crates/x/src/lib.rs:1: 2 types answer 2 moves alike under one renaming: x::A at crates/x/src/lib.rs:1, x::B at crates/x/src/lib.rs:2; one generic over 2 type parameters covers them; write the shared structure once"),
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

#[test]
fn every_pair_of_operations_is_held_to_its_laws_not_only_the_first() {
    assert_eq!(Parallel::with_laws(TWO_OPERATIONS, BOTH_MATCH).matches("alike under one renaming").count(), 1);
    assert_eq!(Parallel::with_laws(TWO_OPERATIONS, SECOND_DIFFERS).matches("alike under one renaming").count(), 0);
}

#[test]
fn a_first_type_whose_laws_differ_does_not_hide_the_others() {
    let text = Parallel::with_laws(THREE_OPERATIONS, FIRST_DIFFERS);
    assert!(text.contains("2 types answer 2 moves alike under one renaming: x::B at crates/x/src/lib.rs:2, x::C at crates/x/src/lib.rs:3"), "{text}");
}

laws::table!(
    the_parameters_count_every_place_where_any_member_differs_other_inputs_included,
    Shown::reported::<OnGraph<ParallelStructures>>,
    [
        [THREE_WAYS, "alike under one renaming"] => vec!["crates/x/src/lib.rs:1: 3 types answer 2 moves alike under one renaming: x::A at crates/x/src/lib.rs:1, x::B at crates/x/src/lib.rs:2, x::C at crates/x/src/lib.rs:3; one generic over 3 type parameters covers them; write the shared structure once, as one generic type or trait over the types the renaming swaps, and make each member an instance of it; where they mean different things, give them moves of their own"],
        [RENAMED_INPUTS, "alike under one renaming"] => vec!["crates/x/src/lib.rs:1: 2 types answer 2 moves alike under one renaming: x::A at crates/x/src/lib.rs:1, x::B at crates/x/src/lib.rs:2; one generic over 2 type parameters covers them; write the shared structure once, as one generic type or trait over the types the renaming swaps, and make each member an instance of it; where they mean different things, give them moves of their own"],
    ]
);
