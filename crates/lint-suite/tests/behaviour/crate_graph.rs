use lint_core::{Lint, SourceFile};
use lint_crate_graph::CrateGraph;
use lint_fixtures::{Lines, Sources, Text};

const ONE: &str = "crates/one/src/lib.rs";
const TWO: &str = "crates/two/src/lib.rs";
const FLAGGED: [&str; 1] = ["crates/one/src/lib.rs:1"];
const BIG_RING: usize = LINKS_IN_ROOT * 2 + 1;
const SMALL_RING: usize = 6;
const LINKS_IN_ROOT: usize = 20;
const LONG_CHAIN: usize = 20_000;

struct Link {
    index: usize,
    next: usize,
}

struct Items;

impl Items {
    fn links(size: usize) -> impl Iterator<Item = Link> {
        (0..size).zip((0..size).cycle().skip(1)).map(|(index, next)| Link { index, next })
    }

    fn ring(prefix: &str, size: usize) -> String {
        let lines: Vec<String> = Self::links(size)
            .map(|Link { index, next }| format!("pub struct {prefix}{index}({prefix}{next});\n"))
            .collect();
        lines.concat()
    }

    fn chain(prefix: &str, size: usize) -> String {
        let lines: Vec<String> = (1..size)
            .map(|next| format!("pub struct {prefix}{}({prefix}{next});\n", next - 1))
            .chain(size.checked_sub(1).map(|last| format!("pub struct {prefix}{last};\n")))
            .collect();
        lines.concat()
    }

    fn ring_with_first_item_in(before: &str, after: &str, size: usize) -> String {
        let rest: Vec<String> =
            Items::links(size).skip(1).map(|Link { index, next }| format!("pub struct A{index}(A{next});\n")).collect();
        format!("{before}pub struct A0(A1);\n{after}{}", rest.concat())
    }

    fn flagged(files: &[SourceFile]) -> Lines {
        Sources::lines(&CrateGraph::report(files))
    }
}

#[test]
fn one_big_cycle_in_a_crate_is_flagged() {
    let big = Sources::file(ONE, Text::new(&Items::ring("A", BIG_RING)));
    let small = Sources::file(ONE, Text::new(&Items::ring("A", SMALL_RING)));
    assert_eq!(Items::flagged(&[big]), Lines::from(FLAGGED));
    assert_eq!(Items::flagged(&[small]), Lines::default());
}

#[test]
fn each_crate_is_named_once_per_cycle() {
    let one = Sources::file(ONE, Text::new(&Items::ring("A", BIG_RING)));
    let two = Sources::file(TWO, Text::new(&Items::ring("A", BIG_RING)));
    assert_eq!(Items::flagged(&[one, two]), Lines::from(["crates/one/src/lib.rs:1", "crates/two/src/lib.rs:1"]));
}

#[test]
fn a_cycle_in_test_code_does_not_count() {
    let in_module =
        Sources::file(ONE, Text::new(&format!("#[cfg(test)]\nmod tests {{\n{}}}\n", Items::ring("A", BIG_RING))));
    assert_eq!(Items::flagged(&[in_module]), Lines::default());
}

#[test]
fn a_long_chain_is_judged_without_exhausting_the_stack() {
    assert_eq!(Items::flagged(&[Sources::file(ONE, Text::new(&Items::chain("A", LONG_CHAIN)))]), Lines::default());
}

#[test]
fn a_cycle_closed_through_impl_bodies_across_files_is_flagged() {
    let link = |Link { index, next }: Link| {
        format!("pub struct R{index};\nimpl R{index} {{\n    pub fn f() {{\n        R{next}::f();\n    }}\n}}\n")
    };
    let first: Vec<String> = Items::links(BIG_RING).take(LINKS_IN_ROOT).map(link).collect();
    let second: Vec<String> = Items::links(BIG_RING).skip(LINKS_IN_ROOT).map(link).collect();
    let root = Sources::file(ONE, Text::new(&format!("mod other;\n{}", first.concat())));
    let other = Sources::file("crates/one/src/other.rs", Text::new(&second.concat()));
    assert_eq!(Items::flagged(&[root, other]), Lines::from(FLAGGED));
}

#[test]
fn a_cycle_through_a_trait_a_fn_and_a_static_is_flagged() {
    let open: Vec<String> = (1..BIG_RING).map(|next| format!("pub struct A{}(A{next});\n", next - 1)).collect();
    let closing = format!(
        "pub struct A{}(Box<dyn T>);\npub trait T {{\n    fn f(&self) {{\n        g();\n    }}\n}}\npub fn g() {{\n    \
         let _ = H;\n}}\npub static H: A0 = A0;\n",
        BIG_RING - 1
    );
    assert_eq!(
        Items::flagged(&[Sources::file(ONE, Text::new(&format!("{}{closing}", open.concat())))]),
        Lines::from(FLAGGED)
    );
}

#[test]
fn a_cycle_inside_an_inline_module_is_flagged() {
    let module = format!("pub mod inner {{\n{}}}\n", Items::ring("A", BIG_RING));
    assert_eq!(Items::flagged(&[Sources::file(ONE, Text::new(&module))]), Lines::from(FLAGGED));
}

#[test]
fn a_link_closed_only_in_a_test_impl_does_not_count() {
    let open = Items::chain("A", BIG_RING);
    let closing = format!("#[cfg(test)]\nimpl A{} {{\n    pub fn f() {{\n        A0::f();\n    }}\n}}\n", BIG_RING - 1);
    assert_eq!(Items::flagged(&[Sources::file(ONE, Text::new(&format!("{open}{closing}")))]), Lines::default());
}

#[test]
fn a_cycle_closed_through_a_type_alias_path_is_flagged() {
    let link = |Link { index, next }: Link| {
        let called = Some(next)
            .filter(|candidate| *candidate != 0)
            .map_or_else(|| "crate::L".to_owned(), |target| format!("crate::R{target}"));
        format!("pub struct R{index};\nimpl R{index} {{\n    pub fn f() {{\n        {called}::f();\n    }}\n}}\n")
    };
    let links: Vec<String> = Items::links(BIG_RING).map(link).collect();
    let source = format!("pub type L = R0;\n{}", links.concat());
    assert_eq!(Items::flagged(&[Sources::file(ONE, Text::new(&source))]), Lines::from(FLAGGED));
}

#[test]
fn items_declared_inside_a_macro_call_take_part_in_a_cycle() {
    let wrapped = Items::ring_with_first_item_in("some_macro! {\n", "}\n", BIG_RING);
    assert_eq!(Items::flagged(&[Sources::file(ONE, Text::new(&wrapped))]), Lines::from(FLAGGED));
}

#[test]
fn items_inside_a_macro_rules_definition_are_not_declared() {
    let defined = Items::ring_with_first_item_in("macro_rules! m {\n    () => {\n", "    };\n}\n", BIG_RING);
    assert_eq!(Items::flagged(&[Sources::file(ONE, Text::new(&defined))]), Lines::default());
}
