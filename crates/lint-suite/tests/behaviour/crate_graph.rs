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
        let lines: Vec<String> = (0..size)
            .map(|index| {
                if index + 1 < size {
                    format!("pub struct {prefix}{index}({prefix}{});\n", index + 1)
                } else {
                    format!("pub struct {prefix}{index};\n")
                }
            })
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
