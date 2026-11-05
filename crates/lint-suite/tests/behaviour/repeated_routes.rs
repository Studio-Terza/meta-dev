use lint_core::Lint;
use lint_fixtures::{Sources, Text};
use lint_transition_graph::RepeatedRoutes;

use crate::shown::OnGraph;

struct Routes;

impl Routes {
    fn of(text: &&str) -> String {
        OnGraph::<RepeatedRoutes>::report(&[Sources::file("crates/x/src/lib.rs", Text::new(text))]).to_string()
    }

    fn walked(text: &&str) -> usize {
        Self::of(text).matches("is walked in").count()
    }
}

const TYPES: &str = "struct A;\nstruct B;\nstruct C;\nstruct D;\nstruct E;\nstruct F;\nimpl A {\n    fn to_b(self) -> B {}\n}\nimpl B {\n    fn to_c(self) -> C {}\n}\nimpl C {\n    fn to_d(self) -> D {}\n}\n";
const TWICE: &str = "struct A;\nstruct B;\nstruct C;\nstruct D;\nstruct E;\nstruct F;\nimpl A {\n    fn to_b(self) -> B {}\n}\nimpl B {\n    fn to_c(self) -> C {}\n}\nimpl C {\n    fn to_d(self) -> D {}\n}\nimpl E {\n    fn first(self, a: A) -> D {\n        a.to_b().to_c().to_d()\n    }\n}\nimpl F {\n    fn second(self, a: A) -> D {\n        a.to_b().to_c().to_d()\n    }\n}\n";
const JOINED: &str = "struct A;\nstruct B;\nstruct C;\nstruct D;\nstruct E;\nstruct F;\nimpl A {\n    fn to_b(self) -> B {}\n    fn straight(self) -> D {}\n}\nimpl B {\n    fn to_c(self) -> C {}\n}\nimpl C {\n    fn to_d(self) -> D {}\n}\nimpl E {\n    fn first(self, a: A) -> D {\n        a.to_b().to_c().to_d()\n    }\n}\nimpl F {\n    fn second(self, a: A) -> D {\n        a.to_b().to_c().to_d()\n    }\n}\n";
const SHORT: &str = "struct A;\nstruct B;\nstruct C;\nstruct E;\nstruct F;\nimpl A {\n    fn to_b(self) -> B {}\n}\nimpl B {\n    fn to_c(self) -> C {}\n}\nimpl E {\n    fn first(self, a: A) -> C {\n        a.to_b().to_c()\n    }\n}\nimpl F {\n    fn second(self, a: A) -> C {\n        a.to_b().to_c()\n    }\n}\n";
const ONCE: &str = "struct A;\nstruct B;\nstruct C;\nstruct D;\nstruct E;\nimpl A {\n    fn to_b(self) -> B {}\n}\nimpl B {\n    fn to_c(self) -> C {}\n}\nimpl C {\n    fn to_d(self) -> D {}\n}\nimpl E {\n    fn first(self, a: A) -> D {\n        a.to_b().to_c().to_d()\n    }\n}\n";
const SHARED_NAME: &str = "struct A;\nstruct B;\nstruct C;\nstruct D;\nstruct E;\nstruct F;\nstruct G;\nimpl A {\n    fn step(self) -> B {}\n}\nimpl B {\n    fn to_c(self) -> C {}\n}\nimpl C {\n    fn to_d(self) -> D {}\n}\nimpl G {\n    fn step(self) -> A {}\n}\nimpl E {\n    fn first(self, a: A) -> D {\n        a.step().to_c().to_d()\n    }\n}\nimpl F {\n    fn second(self, a: A) -> D {\n        a.step().to_c().to_d()\n    }\n}\n";
const STD_RECEIVER: &str = "struct A;\nstruct B;\nstruct C;\nstruct D;\nstruct E;\nstruct F;\nimpl A {\n    fn push(self) -> B {}\n}\nimpl B {\n    fn extend(self) -> C {}\n}\nimpl C {\n    fn clear(self) -> D {}\n}\nimpl E {\n    fn first(self, v: Vec<u8>) {\n        v.push();\n        v.extend();\n        v.clear();\n    }\n}\nimpl F {\n    fn second(self, v: Vec<u8>) {\n        v.push();\n        v.extend();\n        v.clear();\n    }\n}\n";
const OTHER_RECEIVER: &str = "struct A;\nstruct B;\nstruct C;\nstruct D;\nstruct E;\nstruct F;\nstruct X;\nimpl A {\n    fn to_b(self) -> B {}\n}\nimpl B {\n    fn to_c(self) -> C {}\n}\nimpl C {\n    fn to_d(self) -> D {}\n}\nimpl E {\n    fn first(self, x: X) -> D {\n        x.to_b().to_c().to_d()\n    }\n}\nimpl F {\n    fn second(self, x: X) -> D {\n        x.to_b().to_c().to_d()\n    }\n}\n";
const THROUGH_SELF_AND_PATHS: &str = "struct A;\nstruct B;\nstruct C;\nstruct D;\nstruct F;\nimpl A {\n    fn to_b(self) -> B {}\n    fn first(self) -> D {\n        C::to_d(B::to_c(self.to_b()))\n    }\n}\nimpl B {\n    fn to_c(self) -> C {}\n}\nimpl C {\n    fn to_d(self) -> D {}\n}\nimpl F {\n    fn second(self, a: A) -> D {\n        C::to_d(B::to_c(A::to_b(a)))\n    }\n}\n";
const MODULE_PATHS: &str = "struct A;\nstruct B;\nstruct C;\nstruct D;\nstruct E;\nstruct F;\nmod steps {\n    pub fn to_b(a: A) -> B {}\n    pub fn to_c(b: B) -> C {}\n    pub fn to_d(c: C) -> D {}\n}\nimpl E {\n    fn first(self, a: A) -> D {\n        steps::to_d(steps::to_c(steps::to_b(a)))\n    }\n}\nimpl F {\n    fn second(self, a: A) -> D {\n        steps::to_d(steps::to_c(steps::to_b(a)))\n    }\n}\n";
const BORROWED_PARAMETER: &str = "struct A;\nstruct B;\nstruct C;\nstruct D;\nstruct E;\nstruct F;\nimpl A {\n    fn to_b(&self) -> B {}\n}\nimpl B {\n    fn to_c(self) -> C {}\n}\nimpl C {\n    fn to_d(self) -> D {}\n}\nimpl E {\n    fn first(self, a: &A) -> D {\n        a.to_b().to_c().to_d()\n    }\n}\nimpl F {\n    fn second(self, a: &A) -> D {\n        a.to_b().to_c().to_d()\n    }\n}\n";
const THROUGH_SELF_PATHS: &str = "struct A;\nstruct B;\nstruct C;\nstruct D;\nstruct E;\nstruct F;\nimpl A {\n    fn to_b(self) -> B {}\n    fn first(self) -> D {\n        C::to_d(B::to_c(Self::to_b(self)))\n    }\n    fn second(self) -> D {\n        C::to_d(B::to_c(Self::to_b(self)))\n    }\n}\nimpl B {\n    fn to_c(self) -> C {}\n}\nimpl C {\n    fn to_d(self) -> D {}\n}\n";
const WRAPPED_RESULT: &str = "struct A;\nstruct B;\nstruct C;\nstruct D;\nstruct E;\nstruct F;\nimpl A {\n    fn items(self) -> Vec<B> {}\n}\nimpl B {\n    fn len(self) -> C {}\n}\nimpl C {\n    fn to_d(self) -> D {}\n}\nimpl E {\n    fn first(self, a: A) -> D {\n        a.items().len().to_d()\n    }\n}\nimpl F {\n    fn second(self, a: A) -> D {\n        a.items().len().to_d()\n    }\n}\n";
const REBOUND_PARAMETER: &str = "struct A;\nstruct B;\nstruct C;\nstruct D;\nstruct E;\nstruct F;\nstruct X;\nimpl A {\n    fn to_b(self) -> B {}\n}\nimpl B {\n    fn to_c(self) -> C {}\n}\nimpl C {\n    fn to_d(self) -> D {}\n}\nimpl E {\n    fn first(self, a: A, xs: Vec<X>) {\n        xs.into_iter().map(|a| a.to_b().to_c().to_d());\n    }\n}\nimpl F {\n    fn second(self, a: A, xs: Vec<X>) {\n        xs.into_iter().map(|a| a.to_b().to_c().to_d());\n    }\n}\n";
const UNRECORDED_RECEIVER: &str = "struct A;\nstruct B;\nstruct C;\nstruct D;\nstruct E;\nstruct F;\nstruct G;\nimpl A {\n    fn to_b(self) -> B {}\n}\nimpl B {\n    fn to_c(self) -> C {}\n}\nimpl C {\n    fn to_d(self) -> D {}\n}\nimpl E {\n    fn first(self, a: A, g: G) {\n        a.to_b();\n        (g.make)(1).to_c().to_d();\n    }\n}\nimpl F {\n    fn second(self, a: A, g: G) {\n        a.to_b();\n        (g.make)(1).to_c().to_d();\n    }\n}\n";
const SELF_TYPED_PARAMETER: &str = "struct A;\nstruct B;\nstruct C;\nstruct D;\nstruct E;\nstruct F;\nimpl A {\n    fn to_b(self) -> B {}\n    fn first(self, other: Self) -> D {\n        other.to_b().to_c().to_d()\n    }\n    fn second(self, other: &Self) -> D {\n        other.to_b().to_c().to_d()\n    }\n}\nimpl B {\n    fn to_c(self) -> C {}\n}\nimpl C {\n    fn to_d(self) -> D {}\n}\n";
const PATH_CALL_RECEIVER: &str = "struct A;\nstruct B;\nstruct C;\nstruct D;\nstruct E;\nstruct F;\nimpl A {\n    fn to_b(self) -> B {}\n}\nimpl B {\n    fn to_c(self) -> C {}\n}\nimpl C {\n    fn to_d(self) -> D {}\n}\nimpl E {\n    fn first(self, a: A) -> D {\n        A::to_b(a).to_c().to_d()\n    }\n}\nimpl F {\n    fn second(self, a: A) -> D {\n        A::to_b(a).to_c().to_d()\n    }\n}\n";
const LONGER: &str = "struct A;\nstruct B;\nstruct C;\nstruct D;\nstruct H;\nstruct E;\nstruct F;\nimpl A {\n    fn to_b(self) -> B {}\n}\nimpl B {\n    fn to_c(self) -> C {}\n}\nimpl C {\n    fn to_d(self) -> D {}\n}\nimpl D {\n    fn to_h(self) -> H {}\n}\nimpl E {\n    fn first(self, a: A) -> H {\n        a.to_b().to_c().to_d().to_h()\n    }\n}\nimpl F {\n    fn second(self, a: A) -> H {\n        a.to_b().to_c().to_d().to_h()\n    }\n}\n";

const LOOPING: &str = "struct A;\nstruct E;\nstruct F;\nimpl A {\n    fn step(self) -> A {}\n}\nimpl E {\n    fn first(self, a: A) -> A {\n        a.step().step().step()\n    }\n}\nimpl F {\n    fn second(self, a: A) -> A {\n        a.step().step().step()\n    }\n}\n";

laws::table!(
    a_route_is_reported_once_it_has_three_hops_in_two_places_each_resolved_to_one_edge,
    Routes::walked,
    [
        TYPES => 0,
        TWICE => 1,
        SHORT => 0,
        ONCE => 0,
        SHARED_NAME => 1,
        STD_RECEIVER => 0,
        OTHER_RECEIVER => 0,
        THROUGH_SELF_AND_PATHS => 1,
        MODULE_PATHS => 1,
        BORROWED_PARAMETER => 1,
        THROUGH_SELF_PATHS => 1,
        WRAPPED_RESULT => 0,
        REBOUND_PARAMETER => 0,
        UNRECORDED_RECEIVER => 0,
        SELF_TYPED_PARAMETER => 1,
        PATH_CALL_RECEIVER => 1,
        LONGER => 1,
        LOOPING => 0,
    ]
);

#[test]
fn a_route_names_its_hops_and_its_other_places() {
    let text = Routes::of(&TWICE);
    assert!(
        text.contains("crates/x/src/lib.rs:17: a route of 3 hops is walked in 2 places: x::A::to_b, x::B::to_c, x::C::to_d; also at crates/x/src/lib.rs:22; callee names differing: 0; name the route once"),
        "{text}"
    );
}

#[test]
fn a_route_whose_ends_one_edge_already_joins_says_so() {
    let text = Routes::of(&JOINED);
    assert!(text.contains("; x::A::straight already joins its ends; "), "{text}");
}

#[test]
fn the_longest_route_is_kept_and_the_routes_inside_it_are_dropped() {
    let text = Routes::of(&LONGER);
    assert!(text.contains("a route of 4 hops is walked in 2 places"), "{text}");
}

#[test]
fn the_tally_counts_bodies_calls_and_resolved_calls() {
    let text = Routes::of(&TWICE);
    assert!(
        text.contains("workspace: 5 written bodies make 6 calls, 6 resolved to a workspace edge, share unresolved 0.000; 1 repeated routes"),
        "{text}"
    );
}
