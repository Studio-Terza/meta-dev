use lint_core::Lint;
use lint_fixtures::{Located, Source, Sources, Text};
use lint_transition_graph::{Bisimilarity, CommutingTriangles, HiddenStates, LiveNodes, RepeatedMotifs, Tangles};

struct Graphs;

impl Graphs {
    fn of<Report: Lint>(text: &str) -> String {
        Sources::report::<Report>(&[Source::new(
            Located::new("crates/x/src/lib.rs"),
            Text::new(Box::leak(text.to_owned().into_boxed_str())),
        )])
        .to_string()
    }

    fn shows(case: &[&str]) -> bool {
        match case {
            ["hidden", text, expected] => Self::of::<HiddenStates>(text).contains(expected),
            ["live", text, expected] => Self::of::<LiveNodes>(text).contains(expected),
            ["bisimilar", text, expected] => Self::of::<Bisimilarity>(text).contains(expected),
            ["triangle", text, expected] => Self::of::<CommutingTriangles>(text).contains(expected),
            ["motif", text, expected] => Self::of::<RepeatedMotifs>(text).contains(expected),
            ["tangle", text, expected] => Self::of::<Tangles>(text).contains(expected),
            _ => false,
        }
    }
}

const BRANCHY: &str = "struct A;\nstruct B;\nimpl A {\n    fn b(&self) -> B {\n        if p { q } else if r { s } else if t { u } else if v { w } else if x { y } else { z }\n    }\n    fn small(&self) -> B {\n        if p { q } else { z }\n    }\n}\n";
const LIVE: &str = "struct A;\nstruct B;\nstruct C;\nstruct D;\nimpl A {\n    fn new(raw: String) -> A {}\n    fn b(&self) -> B {}\n}\nimpl D {\n    fn a(&self) -> A {}\n}\n";
const ALIKE: &str = "struct A;\nstruct B;\nstruct X;\nstruct Y;\nstruct P;\nstruct Q;\nimpl A {\n    fn x(&self) -> X {}\n    fn shown(&self) -> String {}\n}\nimpl B {\n    fn y(&self) -> Y {}\n    fn shown(&self) -> String {}\n}\nimpl P {\n    fn x(&self) -> X {}\n}\nimpl Q {\n    fn x(&self) -> X {}\n}\n";
const TRIANGLE: &str = "struct A;\nstruct B;\nstruct C;\nimpl A {\n    fn c(&self) -> C {}\n    fn b(&self) -> B {}\n}\nimpl B {\n    fn c(&self) -> C {}\n}\n";
const MOTIF: &str = "struct A;\nstruct B;\nstruct C;\nstruct D;\nstruct E;\nstruct F;\nimpl A {\n    fn read(&self) -> B {}\n}\nimpl B {\n    fn parse(&self) -> C {}\n}\nimpl D {\n    fn read(&self) -> E {}\n}\nimpl E {\n    fn parse(&self) -> F {}\n}\n";
const CYCLES: &str = "struct A;\nstruct B;\nstruct C;\nstruct D;\nstruct E;\nstruct F;\nstruct G;\nimpl A {\n    fn b(&self) -> B {}\n}\nimpl B {\n    fn c(&self) -> C {}\n}\nimpl C {\n    fn a(&self) -> A {}\n}\nimpl From<D> for E {\n    fn from(d: D) -> E {}\n}\nimpl From<E> for D {\n    fn from(e: E) -> D {}\n}\nimpl F {\n    fn g(&self) -> G {}\n    fn held(&self) -> &G { &self.g }\n}\nimpl G {\n    fn f(&self) -> F {}\n}\n";
const PROPAGATING: &str = "struct A;\nstruct B;\nimpl A {\n    fn b(&self) -> B {\n        p()?;\n        p()?;\n        p()?;\n        p()?;\n        p()?;\n        p()?;\n        p()?;\n        p()?;\n        B\n    }\n}\n";
const PAIRED: &str = "struct S;\nstruct N;\nstruct W;\nstruct C;\nimpl W {\n    fn new(s: S, n: N) -> W {}\n    fn s(&self) -> S {}\n    fn c(&self) -> C {}\n}\nimpl S {\n    fn c(&self) -> C {}\n}\n";
const JOINED: &str = "struct S;\nstruct N;\nstruct W;\nstruct C;\nimpl W {\n    fn new(s: S, n: N) -> W {}\n    fn c(&self) -> C {}\n}\nimpl C {\n    fn joined(s: S, n: N) -> C {}\n}\n";
const UNCHAINED: &str = "struct A;\nstruct B;\nstruct C;\nstruct D;\nstruct E;\nstruct F;\nstruct G;\nstruct H;\nimpl B {\n    fn read(a: A, g: G) -> B {}\n}\nimpl C {\n    fn parse(b: B, g: G) -> C {}\n}\nimpl E {\n    fn read(d: D, h: H) -> E {}\n}\nimpl F {\n    fn parse(e: E, a: A) -> F {}\n}\n";
const INNER: &str = "struct A;\nstruct B;\nstruct C;\nimpl A {\n    fn b(&self) -> B {}\n}\nimpl B {\n    fn c(&self) -> C {}\n}\nimpl C {\n    fn joined(a: A, b: B) -> C {}\n}\n";
const CHAINED: &str = "struct A;\nstruct B;\nstruct C;\nstruct D;\nstruct E;\nstruct F;\nstruct G;\nstruct H;\nimpl B {\n    fn read(a: A, g: G) -> B {}\n}\nimpl C {\n    fn parse(b: B, g: G) -> C {}\n}\nimpl E {\n    fn read(d: D, h: H) -> E {}\n}\nimpl F {\n    fn parse(e: E, h: H) -> F {}\n}\n";
const STARVED: &str = "struct A;\nstruct B;\nstruct C;\nimpl A {\n    fn new(raw: String) -> A {}\n}\nimpl B {\n    fn shown(&self) -> String {}\n}\nimpl C {\n    fn joined(a: A, b: B) -> C {}\n    fn shown(&self) -> String {}\n}\n";
const SHOWN: &str = "use std::fmt;\nstruct A;\nstruct B;\nimpl fmt::Display for A {\n    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {}\n}\nimpl fmt::Display for B {\n    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {}\n}\nimpl From<A> for String {\n    fn from(a: A) -> String {}\n}\nimpl From<B> for String {\n    fn from(b: B) -> String {}\n}\n";
const PARTNERED: &str = "struct A;\nstruct B;\nstruct X;\nstruct P;\nimpl A {\n    fn x(&self, p: &P) -> X {}\n    fn shown(&self) -> String {}\n}\nimpl B {\n    fn x(&self, p: &P) -> X {}\n    fn shown(&self) -> String {}\n}\n";
const UNPARTNERED: &str = "struct A;\nstruct B;\nstruct X;\nstruct P;\nstruct Q;\nimpl A {\n    fn x(&self, p: &P) -> X {}\n    fn shown(&self) -> String {}\n}\nimpl B {\n    fn x(&self, q: &Q) -> X {}\n    fn shown(&self) -> String {}\n}\nimpl Q {\n    fn shown(&self) -> String {}\n}\n";
const LIFTED: &str =
    "struct A;\nstruct B;\nimpl A {\n    fn b(&self) -> &B { &self.b }\n}\nimpl B {\n    fn a(&self) -> A {}\n}\n";

laws::table!(
    graph_lints_name_hidden_states_dead_types_bisimilar_types_triangles_motifs_and_cycles,
    Graphs::shows,
    [
        ["hidden", BRANCHY, "crates/x/src/lib.rs:4: x::A::b branches 6 ways, 5 unnamed intermediate states; name the intermediate states as an enum or a typestate"] => true,
        ["hidden", BRANCHY, "crates/x/src/lib.rs:4: workspace: 2 written edges, 1 branch six ways or more, holding 5 unnamed intermediate states"] => true,
        ["hidden", BRANCHY, "x::A::small"] => false,
        ["hidden", PROPAGATING, "crates/x/src/lib.rs:4: workspace: 1 written edges, 0 branch six ways or more, holding 0 unnamed intermediate states"] => true,
        ["hidden", PROPAGATING, "x::A::b branches"] => false,
        ["live", LIVE, "crates/x/src/lib.rs:1: workspace: 4 domain types, 2 not reached from a boundary entry, 2 taken by no edge"] => true,
        ["live", LIVE, "crates/x/src/lib.rs:2: x::B: no edge takes it; consume it through an edge or a port"] => true,
        ["live", LIVE, "crates/x/src/lib.rs:3: x::C: no edge reaches it from a boundary entry and no edge takes it"] => true,
        ["live", LIVE, "crates/x/src/lib.rs:4: x::D: no edge reaches it from a boundary entry; build it"] => true,
        ["live", LIVE, "x::A:"] => false,
        ["live", STARVED, "crates/x/src/lib.rs:1: workspace: 3 domain types, 2 not reached from a boundary entry, 0 taken by no edge"] => true,
        ["live", STARVED, "crates/x/src/lib.rs:3: x::C: no edge reaches it from a boundary entry; build it"] => true,
        ["bisimilar", ALIKE, "crates/x/src/lib.rs:1: 2 types behave alike, each answering written to x::X, written to String: x::A at crates/x/src/lib.rs:1, x::B at crates/x/src/lib.rs:2; merge the group"] => true,
        ["bisimilar", ALIKE, "crates/x/src/lib.rs:1: workspace: 6 domain types, 1 groups of bisimilar types holding 2; set aside as grouped under their interface: 0 interface nodes with 0 fillers"] => true,
        ["bisimilar", SHOWN, "crates/x/src/lib.rs:2: workspace: 2 domain types, 0 groups of bisimilar types holding 0; set aside as grouped under their interface: 2 interface nodes with 4 fillers"] => true,
        ["bisimilar", SHOWN, "behave alike"] => false,
        ["bisimilar", PARTNERED, "crates/x/src/lib.rs:1: 2 types behave alike, each answering written to x::X with x::P, written to String: x::A at crates/x/src/lib.rs:1, x::B at crates/x/src/lib.rs:2"] => true,
        ["bisimilar", UNPARTNERED, "x::A at"] => false,
        ["bisimilar", ALIKE, "x::P at"] => false,
        ["bisimilar", TRIANGLE, "behave alike"] => false,
        ["triangle", TRIANGLE, "crates/x/src/lib.rs:5: x::A -> x::C by x::A::c also goes 1 path(s): through x::B by x::A::b at crates/x/src/lib.rs:6 and x::B::c at crates/x/src/lib.rs:9"] => true,
        ["triangle", TRIANGLE, "crates/x/src/lib.rs:1: workspace: 1 written edges close a triangle, over 1 paths of two edges"] => true,
        ["triangle", MOTIF, "also goes"] => false,
        ["triangle", PAIRED, "crates/x/src/lib.rs:1: workspace: 1 written edges close a triangle, over 1 paths of two edges"] => true,
        ["triangle", PAIRED, "x::S -> x::C"] => false,
        ["triangle", INNER, "crates/x/src/lib.rs:1: workspace: 0 written edges close a triangle, over 0 paths of two edges"] => true,
        ["triangle", JOINED, "crates/x/src/lib.rs:10: (x::N, x::S) -> x::C by x::C::joined also goes 1 path(s): through x::W by x::W::new at crates/x/src/lib.rs:6 and x::W::c at crates/x/src/lib.rs:7"] => true,
        ["motif", MOTIF, "crates/x/src/lib.rs:8: chain read then parse repeats over 2 disjoint type sets: x::A -> x::B -> x::C at crates/x/src/lib.rs:8, crates/x/src/lib.rs:11; x::D -> x::E -> x::F at crates/x/src/lib.rs:14, crates/x/src/lib.rs:17"] => true,
        ["motif", MOTIF, "crates/x/src/lib.rs:1: workspace: 1 repeated motifs over 2 disjoint type sets"] => true,
        ["motif", TRIANGLE, "repeats over"] => false,
        ["motif", PAIRED, "repeats over"] => false,
        ["motif", CHAINED, "crates/x/src/lib.rs:10: chain read then parse repeats over 2 disjoint type sets: (x::A, x::G) -> x::B -> x::C at crates/x/src/lib.rs:10, crates/x/src/lib.rs:13; (x::D, x::H) -> x::E -> x::F at crates/x/src/lib.rs:16, crates/x/src/lib.rs:19"] => true,
        ["motif", UNCHAINED, "crates/x/src/lib.rs:1: workspace: 0 repeated motifs over 0 disjoint type sets"] => true,
        ["tangle", CYCLES, "crates/x/src/lib.rs:1: tangle of 3 types: x::A at crates/x/src/lib.rs:1, x::B at crates/x/src/lib.rs:2, x::C at crates/x/src/lib.rs:3; joined by 3 edges: x::A::b at crates/x/src/lib.rs:9"] => true,
        ["tangle", CYCLES, "crates/x/src/lib.rs:4: round trip of 2 types: x::D at crates/x/src/lib.rs:4, x::E at crates/x/src/lib.rs:5; joined by 2 edges"] => true,
        ["tangle", CYCLES, "crates/x/src/lib.rs:6: tangle of 2 types: x::F at crates/x/src/lib.rs:6, x::G at crates/x/src/lib.rs:7; joined by 2 edges: x::F::g at crates/x/src/lib.rs:24, x::G::f at crates/x/src/lib.rs:28"] => true,
        ["tangle", CYCLES, "crates/x/src/lib.rs:1: workspace: 2 tangles holding 5 types, 1 round trips"] => true,
        ["tangle", LIFTED, "tangle of"] => false,
        ["tangle", MOTIF, "tangle of"] => false,
        ["tangle", PAIRED, "crates/x/src/lib.rs:1: workspace: 0 tangles holding 0 types, 0 round trips"] => true,
    ]
);
