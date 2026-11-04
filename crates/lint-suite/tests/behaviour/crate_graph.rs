use lint_core::{Lint, SourceFile};
use lint_crate_graph::CrateGraph;
use lint_fixtures::{Lines, Sources, Text};

const ONE: &str = "crates/one/src/lib.rs";
const TWO: &str = "crates/two/src/lib.rs";
const FLAGGED: [&str; 1] = ["crates/one/src/lib.rs:1"];
const CLEAN: [&str; 0] = [];
const BIG_RING: usize = 41;
const SMALL_RING: usize = 6;
const LONG_CHAIN: usize = 20_000;

struct Items;

impl Items {
    fn ring(prefix: &str, size: usize) -> String {
        let lines: Vec<String> =
            (0..size).map(|index| format!("pub struct {prefix}{index}({prefix}{});\n", (index + 1) % size)).collect();
        lines.concat()
    }

    fn chain(prefix: &str, size: usize) -> String {
        let lines: Vec<String> = (1..size)
            .map(|next| format!("pub struct {prefix}{}({prefix}{next});\n", next - 1))
            .chain(size.checked_sub(1).map(|last| format!("pub struct {prefix}{last};\n")))
            .collect();
        lines.concat()
    }

    fn flagged(files: &[SourceFile]) -> Lines {
        Sources::lines(&CrateGraph::report(files))
    }
}

#[test]
fn one_big_cycle_in_a_crate_is_flagged() {
    let big = Sources::file(ONE, Text::new(&Items::ring("A", BIG_RING)));
    let small = Sources::file(ONE, Text::new(&Items::ring("A", SMALL_RING)));
    assert_eq!(Items::flagged(&[big]), FLAGGED);
    assert_eq!(Items::flagged(&[small]), CLEAN);
}

#[test]
fn each_crate_is_named_once_per_cycle() {
    let one = Sources::file(ONE, Text::new(&Items::ring("A", BIG_RING)));
    let two = Sources::file(TWO, Text::new(&Items::ring("A", BIG_RING)));
    assert_eq!(Items::flagged(&[one, two]), ["crates/one/src/lib.rs:1", "crates/two/src/lib.rs:1"]);
}

#[test]
fn a_cycle_in_test_code_does_not_count() {
    let in_module =
        Sources::file(ONE, Text::new(&format!("#[cfg(test)]\nmod tests {{\n{}}}\n", Items::ring("A", BIG_RING))));
    assert_eq!(Items::flagged(&[in_module]), CLEAN);
}

#[test]
fn a_long_chain_is_judged_without_exhausting_the_stack() {
    assert_eq!(Items::flagged(&[Sources::file(ONE, Text::new(&Items::chain("A", LONG_CHAIN)))]), CLEAN);
}

#[test]
fn a_cycle_closed_through_impl_bodies_across_files_is_flagged() {
    let link = |index: usize| {
        format!(
            "pub struct R{index};\nimpl R{index} {{\n    pub fn f() {{\n        R{}::f();\n    }}\n}}\n",
            (index + 1) % BIG_RING
        )
    };
    let half = BIG_RING / 2;
    let first: Vec<String> = (0..half).map(link).collect();
    let second: Vec<String> = (half..BIG_RING).map(link).collect();
    let root = Sources::file(ONE, Text::new(&format!("mod other;\n{}", first.concat())));
    let other = Sources::file("crates/one/src/other.rs", Text::new(&second.concat()));
    assert_eq!(Items::flagged(&[root, other]), FLAGGED);
}

#[test]
fn a_cycle_through_a_trait_a_fn_and_a_static_is_flagged() {
    let open: Vec<String> = (1..BIG_RING).map(|next| format!("pub struct A{}(A{next});\n", next - 1)).collect();
    let closing = format!(
        "pub struct A{}(Box<dyn T>);\npub trait T {{\n    fn f(&self) {{\n        g();\n    }}\n}}\npub fn g() {{\n    \
         let _ = H;\n}}\npub static H: A0 = A0;\n",
        BIG_RING - 1
    );
    assert_eq!(Items::flagged(&[Sources::file(ONE, Text::new(&format!("{}{closing}", open.concat())))]), FLAGGED);
}

#[test]
fn a_cycle_inside_an_inline_module_is_flagged() {
    let module = format!("pub mod inner {{\n{}}}\n", Items::ring("A", BIG_RING));
    assert_eq!(Items::flagged(&[Sources::file(ONE, Text::new(&module))]), FLAGGED);
}

#[test]
fn a_link_closed_only_in_a_test_impl_does_not_count() {
    let open = Items::chain("A", BIG_RING);
    let closing = format!("#[cfg(test)]\nimpl A{} {{\n    pub fn f() {{\n        A0::f();\n    }}\n}}\n", BIG_RING - 1);
    assert_eq!(Items::flagged(&[Sources::file(ONE, Text::new(&format!("{open}{closing}")))]), CLEAN);
}

#[test]
fn a_cycle_closed_through_a_type_alias_path_is_flagged() {
    let link = |index: usize| {
        let next = (index + 1) % BIG_RING;
        let called = Some(next)
            .filter(|next| *next != 0)
            .map_or_else(|| "crate::L".to_owned(), |next| format!("crate::R{next}"));
        format!("pub struct R{index};\nimpl R{index} {{\n    pub fn f() {{\n        {called}::f();\n    }}\n}}\n")
    };
    let links: Vec<String> = (0..BIG_RING).map(link).collect();
    let source = format!("pub type L = R0;\n{}", links.concat());
    assert_eq!(Items::flagged(&[Sources::file(ONE, Text::new(&source))]), FLAGGED);
}
