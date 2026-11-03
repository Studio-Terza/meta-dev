use lint_core::Lint;
use lint_fixtures::{Located, Source, Sources, Text};
use lint_layering::Layering;

struct Layered;

impl Layered {
    fn of(sources: &[Source]) -> String {
        Layering::report(&Sources::parsed(sources)).to_string()
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

const FIELDS: &str = "struct A {\n    b: B,\n}\nstruct B {\n    a: Vec<A>,\n}\nstruct C {\n    a: A,\n}\n";
const VARIANTS: &str = "enum A {\n    B(B),\n}\nenum B {\n    A(Box<A>),\n    Leaf,\n}\n";
const BOUNDS: &str = "struct A<T: Into<B>> {\n    t: T,\n}\nstruct B {\n    a: A<u8>,\n}\n";
const RECURSIVE: &str = "enum Tree {\n    Leaf,\n    Node(Box<Tree>, Vec<Tree>),\n}\nstruct Scope<'outer> {\n    outer: Option<&'outer Scope<'outer>>,\n}\n";

laws::table!(
    layering_lists_each_cycle_of_definitions_except_self_recursion_through_a_pointer,
    Layered::shows,
    [
        [FIELDS, "crates/x/src/lib.rs:1: tangle of 2 definitions: x::A at crates/x/src/lib.rs:1, x::B at crates/x/src/lib.rs:4; closed by 2 edges: x::A -> x::B (field) at crates/x/src/lib.rs:2, x::B -> x::A (field) at crates/x/src/lib.rs:5;"] => true,
        [FIELDS, "x::C at"] => false,
        [FIELDS, "pull the shared part into its own type, or make the recursion explicit: a type holding itself through Box, Rc or Arc"] => true,
        [VARIANTS, "x::A -> x::B (variant) at crates/x/src/lib.rs:2, x::B -> x::A (variant) at crates/x/src/lib.rs:5"] => true,
        [BOUNDS, "x::A -> x::B (bound) at crates/x/src/lib.rs:1, x::B -> x::A (field) at crates/x/src/lib.rs:5"] => true,
        [RECURSIVE, "tangle"] => false,
        ["struct A {\n    a: std::marker::PhantomData<A>,\n}\n", "tangle of 1 definitions: x::A at crates/x/src/lib.rs:1; closed by 1 edges: x::A -> x::A (field)"] => true,
        ["struct A {\n    b: std::fmt::B,\n}\nstruct B {\n    a: A,\n}\n", "tangle"] => false,
        ["fn a() {}\n", "tangle"] => false,
    ]
);

#[test]
fn a_qualified_path_resolves_to_the_crate_it_names() {
    let sources = [
        Source::new(Located::new("crates/x/src/lib.rs"), Text::new("struct A {\n    b: y::B,\n}\nstruct B;\n")),
        Source::new(Located::new("crates/y/src/lib.rs"), Text::new("struct B {\n    a: x::A,\n}\n")),
    ];
    let text = Layered::of(&sources);
    assert!(
        text.contains("x::A -> y::B (field) at crates/x/src/lib.rs:2, y::B -> x::A (field) at crates/y/src/lib.rs:2"),
        "{text}"
    );
}

#[test]
fn an_unqualified_name_resolves_to_the_one_other_crate_declaring_it_and_an_ambiguous_one_is_dropped() {
    let x = || Source::new(Located::new("crates/x/src/lib.rs"), Text::new("struct A {\n    b: B,\n}\n"));
    let y = || Source::new(Located::new("crates/y/src/lib.rs"), Text::new("struct B {\n    a: A,\n}\n"));
    let resolved = Layered::of(&[x(), y()]);
    assert!(resolved.contains("x::A -> y::B (field) at crates/x/src/lib.rs:2, y::B -> x::A (field)"), "{resolved}");
    let ambiguous =
        Layered::of(&[x(), y(), Source::new(Located::new("crates/z/src/lib.rs"), Text::new("struct B;\n"))]);
    assert!(!ambiguous.contains("tangle"), "{ambiguous}");
}
