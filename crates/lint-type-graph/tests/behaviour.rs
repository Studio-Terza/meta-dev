use lint_fixtures::{Located, Source, Sources, Text};
use lint_type_graph::TypeGraph;

struct Graphed;

impl Graphed {
    fn of(sources: &[Source]) -> String {
        Sources::report::<TypeGraph>(sources).to_string()
    }

    fn shows(pair: &[&str]) -> bool {
        match pair {
            [text, expected] => {
                let leaked: &'static str = Box::leak((*text).to_owned().into_boxed_str());
                Self::of(&[Source::new(Located::new("crates/x/src/lib.rs"), Text::new(leaked))]).contains(expected)
            }
            _ => false,
        }
    }
}

const CYCLE_OF_THREE: &str = "struct A;\nstruct B;\nstruct C;\nstruct D;\nimpl From<A> for B {}\nimpl From<B> for C {}\nimpl From<C> for A {}\nimpl From<C> for D {}\n";
const EVERY_KIND: &str = "struct A;\nstruct B;\nstruct C;\nstruct D;\nstruct E;\nimpl TryFrom<A> for B {}\nimpl AsRef<C> for B {}\nimpl Into<D> for B {}\nimpl AsMut<E> for B {}\nimpl Borrow<A> for E {}\nimpl TryInto<A> for C {}\n";
const IGNORED: &str = "struct A;\nstruct B;\nimpl std::fmt::Display for A {}\nimpl From<Foreign> for A {}\nimpl From<A> for A {}\nimpl From<B> for A {}\nimpl From<B> for A {}\nimpl AsRef<str> for B {}\n";

laws::table!(
    the_graph_of_a_crate_is_measured_by_nodes_edges_density_and_its_largest_component,
    Graphed::shows,
    [
        ["struct A;\nstruct B;\nimpl From<B> for A {}\nimpl From<A> for B {}\n", "crates/x/src/lib.rs:1: crate x: 2 nodes, 2 edges, density 1.000, largest strongly connected component 2;"] => true,
        ["struct A;\nstruct B;\nimpl From<A> for B {}\n", "2 nodes, 1 edges, density 0.500, largest strongly connected component 1"] => true,
        [EVERY_KIND, "5 nodes, 6 edges, density 0.300"] => true,
        [CYCLE_OF_THREE, "4 nodes, 4 edges, density 0.333, largest strongly connected component 3"] => true,
        ["struct A;\nstruct B;\ntrait T { fn m(&self); }\nimpl T for A {}\nimpl T for B {}\n", "3 nodes, 2 edges, density 0.333, largest strongly connected component 1"] => true,
        [IGNORED, "2 nodes, 1 edges, density 0.500"] => true,
        ["struct A;\n", "1 nodes, 0 edges, density 0.000, largest strongly connected component 1"] => true,
        ["\n\ntrait T {}\nstruct A;\n", "crates/x/src/lib.rs:3: crate x: 2 nodes"] => true,
        ["fn a() {}\n", "crate x"] => false,
    ]
);

#[test]
fn each_crate_is_its_own_graph_and_edges_do_not_cross() {
    let sources = [
        Source::new(Located::new("crates/x/src/lib.rs"), Text::new("struct A;\nimpl From<B> for A {}\n")),
        Source::new(Located::new("crates/y/src/lib.rs"), Text::new("struct B;\nstruct C;\nimpl From<B> for C {}\n")),
    ];
    let text = Graphed::of(&sources);
    assert!(text.contains("crate x: 1 nodes, 0 edges"), "{text}");
    assert!(text.contains("crate y: 2 nodes, 1 edges"), "{text}");
}
