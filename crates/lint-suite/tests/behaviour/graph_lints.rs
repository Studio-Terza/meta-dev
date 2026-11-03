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
const LIVE: &str =
    "struct A;\nstruct B;\nstruct C;\nimpl A {\n    fn new(raw: String) -> A {}\n    fn b(&self) -> B {}\n}\n";
const ALIKE: &str = "struct A;\nstruct B;\nstruct Out;\nimpl A {\n    fn out(&self) -> Out {}\n    fn shown(&self) -> String {}\n}\nimpl B {\n    fn out(&self) -> Out {}\n    fn shown(&self) -> String {}\n}\n";
const TRIANGLE: &str = "struct A;\nstruct B;\nstruct C;\nimpl A {\n    fn c(&self) -> C {}\n    fn b(&self) -> B {}\n}\nimpl B {\n    fn c(&self) -> C {}\n}\n";
const MOTIF: &str = "struct A;\nstruct B;\nstruct C;\nstruct D;\nstruct E;\nstruct F;\nimpl A {\n    fn read(&self) -> B {}\n}\nimpl B {\n    fn parse(&self) -> C {}\n}\nimpl D {\n    fn read(&self) -> E {}\n}\nimpl E {\n    fn parse(&self) -> F {}\n}\n";
const CYCLES: &str = "struct A;\nstruct B;\nstruct C;\nstruct D;\nstruct E;\nimpl A {\n    fn b(&self) -> B {}\n}\nimpl B {\n    fn c(&self) -> C {}\n}\nimpl C {\n    fn a(&self) -> A {}\n}\nimpl From<D> for E {\n    fn from(d: D) -> E {}\n}\nimpl From<E> for D {\n    fn from(e: E) -> D {}\n}\n";

laws::table!(
    graph_lints_name_hidden_states_dead_types_bisimilar_types_triangles_motifs_and_cycles,
    Graphs::shows,
    [
        ["hidden", BRANCHY, "crates/x/src/lib.rs:4: x::A::b branches 6 ways, 5 unnamed intermediate states; name the intermediate states as an enum or a typestate"] => true,
        ["hidden", BRANCHY, "workspace: 2 written edges, 1 branch six ways or more, holding 5 unnamed intermediate states"] => true,
        ["hidden", BRANCHY, "x::A::small"] => false,
        ["live", LIVE, "crates/x/src/lib.rs:3: x::C: no edge reaches it from a boundary entry and no edge takes it"] => true,
        ["live", LIVE, "crates/x/src/lib.rs:2: x::B: no edge takes it; consume it through an edge or a port"] => true,
        ["live", LIVE, "x::A:"] => false,
        ["bisimilar", ALIKE, "2 types behave alike, each answering written to x::Out, written to String: x::A at crates/x/src/lib.rs:1, x::B at crates/x/src/lib.rs:2"] => true,
        ["bisimilar", TRIANGLE, "behave alike"] => false,
        ["triangle", TRIANGLE, "crates/x/src/lib.rs:5: x::A -> x::C by x::A::c also goes 1 path(s): through x::B by x::A::b at crates/x/src/lib.rs:6 and x::B::c at crates/x/src/lib.rs:9"] => true,
        ["triangle", MOTIF, "also goes"] => false,
        ["motif", MOTIF, "chain read then parse repeats over 2 disjoint type sets: x::A -> x::B -> x::C at crates/x/src/lib.rs:8, crates/x/src/lib.rs:11; x::D -> x::E -> x::F at crates/x/src/lib.rs:14, crates/x/src/lib.rs:17"] => true,
        ["motif", TRIANGLE, "repeats over"] => false,
        ["tangle", CYCLES, "tangle of 3 types: x::A at crates/x/src/lib.rs:1, x::B at crates/x/src/lib.rs:2, x::C at crates/x/src/lib.rs:3; joined by 3 edges"] => true,
        ["tangle", CYCLES, "round trip of 2 types: x::D at crates/x/src/lib.rs:4, x::E at crates/x/src/lib.rs:5; joined by 2 edges"] => true,
        ["tangle", CYCLES, "workspace: 1 tangles holding 3 types, 1 round trips"] => true,
        ["tangle", MOTIF, "tangle of"] => false,
    ]
);
