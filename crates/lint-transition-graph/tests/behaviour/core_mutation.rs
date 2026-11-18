use lint_transition_graph::CoreMutation;

use crate::shown::OnGraph;
use lint_fixtures::{Checks, Located, Shown, Source, Sources, Text};

const MUTATION: &str = "struct A;\nimpl A {\n    fn run(&mut self) {}\n    fn fill(&self, out: &mut Vec<A>) {}\n    fn both(&mut self, out: &mut A) {}\n    fn pure(&self) -> A {}\n}\nimpl fmt::Display for A {\n    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {}\n}\nimpl Iterator for A {\n    fn next(&mut self) -> Option<A> {}\n}\nimpl AsMut<A> for B {\n    fn as_mut(&mut self) -> &mut A { &mut self.a }\n}\ntrait Local {\n    fn bump(&mut self);\n}\nimpl Local for A {\n    fn bump(&mut self) {}\n}\nmod ports {\n    fn write(out: &mut A) {}\n}\nmod boundary {\n    fn scan(out: &mut A) {}\n}\nmod store {\n    impl Records {\n        fn insert(&mut self, record: A, f: impl Fn(u8)) -> Option<usize> {}\n        fn merge(&self, into: &mut A) {}\n    }\n}\n";
const PURE: &str = "struct A;\nimpl A {\n    fn pure(&self) -> A {}\n}\nmod ports {\n    fn write(out: &mut A) {}\n}\n";

laws::table!(
    core_edges_taking_mut_are_counted_apart_from_ports_boundaries_stores_and_foreign_trait_signatures,
    Shown::reported::<OnGraph<CoreMutation>>,
    [
        [MUTATION, "crate x:"] => vec!["crates/x/src/lib.rs:3: crate x: 7 core edges take &mut, 5 through &mut self, 2 through a &mut parameter only, 3 with a signature fixed by a foreign trait, 4 to migrate; 1 store operations, x::Records::insert applies x::A, impl Fn -> usize; move each mutation into a store operation, a &mut self method in a store module, or return a new value"],
        [MUTATION, "workspace:"] => vec!["crates/x/src/lib.rs:3: workspace: 7 core edges take &mut, 5 through &mut self, 2 through a &mut parameter only, 3 with a signature fixed by a foreign trait, 4 to migrate; 1 store operations; move each mutation into a store operation, a &mut self method in a store module, or return a new value"],
        [PURE, "crate x"] => Vec::<String>::new(),
        [PURE, "workspace:"] => Vec::<String>::new(),
    ]
);

const PORTS_SOURCE: &str = "pub trait Execute {}\npub type Statements<'run> = dyn Execute + 'run;\n";
const PORT_BOUNDARY: &str = "mod boundary {\n    pub fn read(statements: &mut store_ports::Statements<'_>) {}\n}\n";
const PORT_DIRECT: &str = "mod boundary {\n    pub fn read(statements: &mut dyn store_ports::Execute) {}\n}\n";
const PORT_STORE: &str = "mod store {\n    pub fn read(statements: &mut store_ports::Statements<'_>) {}\n}\n";
const PORT_CORE: &str = "pub fn read(statements: &mut store_ports::Statements<'_>) {}\n";
const PORT_LOCAL: &str = "trait Execute {}\ntype Statements<'run> = dyn Execute + 'run;\nmod boundary {\n    pub fn read(statements: &mut super::Statements<'_>) {}\n}\n";
const PORT_MIXED: &str =
    "mod boundary {\n    pub fn read(statements: &mut store_ports::Statements<'_>, sink: &mut Vec<u8>) {}\n}\n";

const PORTS: Source = Source::new(Located::new("crates/store-ports/src/lib.rs"), Text::new(PORTS_SOURCE));

struct Handles;

impl Handles {
    fn reported(sources: &[Source]) -> Vec<String> {
        <OnGraph<CoreMutation> as Checks>::reported(&Sources::parsed(sources), Shown::listed)
    }
}

laws::table!(
    a_mut_store_ports_statements_parameter_in_a_boundary_module_is_a_store_operation,
    Handles::reported,
    [
        [PORTS, Source::new(Located::new("crates/x/src/lib.rs"), Text::new(PORT_BOUNDARY))] => vec![
            "crates/x/src/lib.rs:2: crate x: 0 core edges take &mut, 0 through &mut self, 0 through a &mut parameter only, 0 with a signature fixed by a foreign trait, 0 to migrate; 1 store operations, x::read applies  -> (); move each mutation into a store operation, a &mut self method in a store module, or return a new value".to_owned(),
            "crates/x/src/lib.rs:2: workspace: 0 core edges take &mut, 0 through &mut self, 0 through a &mut parameter only, 0 with a signature fixed by a foreign trait, 0 to migrate; 1 store operations; move each mutation into a store operation, a &mut self method in a store module, or return a new value".to_owned(),
        ],
    ]
);

laws::table!(
    a_mut_dyn_store_ports_execute_parameter_in_a_boundary_module_is_a_store_operation,
    Handles::reported,
    [
        [PORTS, Source::new(Located::new("crates/x/src/lib.rs"), Text::new(PORT_DIRECT))] => vec![
            "crates/x/src/lib.rs:2: crate x: 0 core edges take &mut, 0 through &mut self, 0 through a &mut parameter only, 0 with a signature fixed by a foreign trait, 0 to migrate; 1 store operations, x::read applies  -> (); move each mutation into a store operation, a &mut self method in a store module, or return a new value".to_owned(),
            "crates/x/src/lib.rs:2: workspace: 0 core edges take &mut, 0 through &mut self, 0 through a &mut parameter only, 0 with a signature fixed by a foreign trait, 0 to migrate; 1 store operations; move each mutation into a store operation, a &mut self method in a store module, or return a new value".to_owned(),
        ],
    ]
);

laws::table!(
    a_mut_store_ports_statements_parameter_in_a_store_module_is_a_store_operation,
    Handles::reported,
    [
        [PORTS, Source::new(Located::new("crates/x/src/lib.rs"), Text::new(PORT_STORE))] => vec![
            "crates/x/src/lib.rs:2: crate x: 0 core edges take &mut, 0 through &mut self, 0 through a &mut parameter only, 0 with a signature fixed by a foreign trait, 0 to migrate; 1 store operations, x::read applies  -> (); move each mutation into a store operation, a &mut self method in a store module, or return a new value".to_owned(),
            "crates/x/src/lib.rs:2: workspace: 0 core edges take &mut, 0 through &mut self, 0 through a &mut parameter only, 0 with a signature fixed by a foreign trait, 0 to migrate; 1 store operations; move each mutation into a store operation, a &mut self method in a store module, or return a new value".to_owned(),
        ],
    ]
);

laws::table!(
    a_mut_store_ports_statements_parameter_in_a_core_module_is_a_core_edge_taking_mut,
    Handles::reported,
    [
        [PORTS, Source::new(Located::new("crates/x/src/lib.rs"), Text::new(PORT_CORE))] => vec![
            "crates/x/src/lib.rs:1: crate x: 1 core edges take &mut, 0 through &mut self, 1 through a &mut parameter only, 0 with a signature fixed by a foreign trait, 1 to migrate; 0 store operations; move each mutation into a store operation, a &mut self method in a store module, or return a new value".to_owned(),
            "crates/x/src/lib.rs:1: workspace: 1 core edges take &mut, 0 through &mut self, 1 through a &mut parameter only, 0 with a signature fixed by a foreign trait, 1 to migrate; 0 store operations; move each mutation into a store operation, a &mut self method in a store module, or return a new value".to_owned(),
        ],
    ]
);

laws::table!(
    a_local_trait_named_execute_is_not_the_store_port,
    Handles::reported,
    [
        [PORTS, Source::new(Located::new("crates/x/src/lib.rs"), Text::new(PORT_LOCAL))] => Vec::<String>::new(),
    ]
);

laws::table!(
    another_mut_parameter_beside_the_store_port_is_not_a_store_operation,
    Handles::reported,
    [
        [PORTS, Source::new(Located::new("crates/x/src/lib.rs"), Text::new(PORT_MIXED))] => Vec::<String>::new(),
    ]
);
