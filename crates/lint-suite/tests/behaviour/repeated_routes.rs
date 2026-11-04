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
const LONGER: &str = "struct A;\nstruct B;\nstruct C;\nstruct D;\nstruct H;\nstruct E;\nstruct F;\nimpl A {\n    fn to_b(self) -> B {}\n}\nimpl B {\n    fn to_c(self) -> C {}\n}\nimpl C {\n    fn to_d(self) -> D {}\n}\nimpl D {\n    fn to_h(self) -> H {}\n}\nimpl E {\n    fn first(self, a: A) -> H {\n        a.to_b().to_c().to_d().to_h()\n    }\n}\nimpl F {\n    fn second(self, a: A) -> H {\n        a.to_b().to_c().to_d().to_h()\n    }\n}\n";

laws::table!(
    a_route_is_reported_once_it_has_three_hops_in_two_places_each_resolved_to_one_edge,
    Routes::walked,
    [
        TYPES => 0,
        TWICE => 1,
        SHORT => 0,
        ONCE => 0,
        SHARED_NAME => 0,
        LONGER => 1,
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
