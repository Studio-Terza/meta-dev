use lint_core::{Lint, SourceFile};
use lint_crate_graph::CrateGraph;
use lint_fixtures::Sources;

const ONE: &str = "crates/one/src/lib.rs";
const TWO: &str = "crates/two/src/lib.rs";
const FLAGGED: [&str; 1] = ["crates/one:1"];
const CLEAN: [&str; 0] = [];
const BIG_RING: usize = 41;
const SMALL_RING: usize = 6;
const GROUP: usize = 5;
const SHORT_GROUP: usize = 4;
const CLIQUE: usize = 6;

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

    fn clique(prefix: &str, size: usize) -> String {
        let lines: Vec<String> = (0..size)
            .map(|index| {
                let later: Vec<String> = (index + 1..size).map(|other| format!("{prefix}{other}")).collect();
                format!("pub struct {prefix}{index}({});\n", later.join(", "))
            })
            .collect();
        lines.concat()
    }

    fn flagged(files: &[SourceFile]) -> Vec<String> {
        Sources::lines(&CrateGraph::report(files))
    }
}

#[test]
fn one_big_cycle_in_a_crate_is_flagged() {
    let big = Sources::file(ONE, &Items::ring("A", BIG_RING));
    let small = Sources::file(ONE, &Items::ring("A", SMALL_RING));
    assert_eq!(Items::flagged(&[big]), FLAGGED);
    assert_eq!(Items::flagged(&[small]), CLEAN);
}

#[test]
fn two_unconnected_groups_are_flagged_unless_one_is_short() {
    let even = Sources::file(ONE, &format!("{}{}", Items::chain("A", GROUP), Items::chain("B", GROUP)));
    let uneven = Sources::file(ONE, &format!("{}{}", Items::chain("A", GROUP), Items::chain("B", SHORT_GROUP)));
    assert_eq!(Items::flagged(&[even]), FLAGGED);
    assert_eq!(Items::flagged(&[uneven]), CLEAN);
}

#[test]
fn loosely_linked_groups_are_flagged_and_tightly_linked_ones_are_not() {
    let linked = format!("{}{}pub struct Bridge(A0, B0);\n", Items::clique("A", CLIQUE), Items::clique("B", CLIQUE));
    let all = "A0, A1, A2, A3, A4, A5, B0, B1, B2, B3, B4, B5";
    let tight = format!("{linked}pub struct Second({all});\npub struct Third({all});\n");
    assert_eq!(Items::flagged(&[Sources::file(ONE, &linked)]), FLAGGED);
    assert_eq!(Items::flagged(&[Sources::file(ONE, &Items::clique("A", CLIQUE))]), CLEAN);
    assert_eq!(Items::flagged(&[Sources::file(ONE, &tight)]), CLEAN);
}

#[test]
fn a_crate_is_judged_on_all_its_files_together() {
    let first = Sources::file(ONE, &format!("mod other;\n{}", Items::chain("A", GROUP)));
    let second = Sources::file("crates/one/src/other.rs", &Items::chain("B", GROUP));
    assert_eq!(Items::flagged(&[first, second]), FLAGGED);
}

#[test]
fn crates_are_judged_apart() {
    let one = Sources::file(ONE, &Items::chain("A", GROUP));
    let two = Sources::file(TWO, &Items::chain("B", GROUP));
    assert_eq!(Items::flagged(&[one, two]), CLEAN);
}

#[test]
fn each_flagged_crate_is_named_once_per_finding() {
    let one = Sources::file(ONE, &format!("{}{}", Items::chain("A", GROUP), Items::chain("B", GROUP)));
    let two = Sources::file(TWO, &format!("{}{}", Items::chain("A", GROUP), Items::chain("B", GROUP)));
    assert_eq!(Items::flagged(&[one, two]), ["crates/one:1", "crates/two:1"]);
}

#[test]
fn test_code_does_not_count() {
    let in_module = Sources::file(
        ONE,
        &format!("{}#[cfg(test)]\nmod tests {{\n{}}}\n", Items::chain("A", GROUP), Items::chain("B", GROUP)),
    );
    let in_target = Sources::file("crates/one/tests/flow.rs", &Items::chain("B", GROUP));
    let shipped = Sources::file(ONE, &Items::chain("A", GROUP));
    assert_eq!(Items::flagged(&[in_module]), CLEAN);
    assert_eq!(Items::flagged(&[shipped, in_target]), CLEAN);
}

#[test]
fn a_name_from_another_crate_is_not_a_link() {
    let join = "pub struct Join(A0, A1, A2, A3, A4, B0, B1, B2, B3, B4);\n";
    let groups = format!("{}{}", Items::chain("A", GROUP), Items::chain("B", GROUP));
    let local = format!("{groups}{join}");
    let imported = format!("use elsewhere::{{A0, A1, A2, A3, A4}};\n{groups}{join}");
    assert_eq!(Items::flagged(&[Sources::file(ONE, &local)]), CLEAN);
    assert_eq!(Items::flagged(&[Sources::file(ONE, &imported)]), FLAGGED);
}
