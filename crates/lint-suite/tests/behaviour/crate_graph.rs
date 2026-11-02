use lint_core::{Lint, SourceFile};
use lint_crate_graph::CrateGraph;
use lint_fixtures::{Lines, Sources, Text};

const ONE: &str = "crates/one/src/lib.rs";
const TWO: &str = "crates/two/src/lib.rs";
const FLAGGED: [&str; 1] = ["crates/one/src/lib.rs:1"];
const CLEAN: [&str; 0] = [];
const BIG_RING: usize = 41;
const SMALL_RING: usize = 6;
const GROUP: usize = 5;
const SHORT_GROUP: usize = 4;
const CLIQUE: usize = 6;
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

    fn clique(prefix: &str, size: usize) -> String {
        let lines: Vec<String> = (0..size)
            .map(|index| {
                let later: Vec<String> = (index + 1..size).map(|other| format!("{prefix}{other}")).collect();
                format!("pub struct {prefix}{index}({});\n", later.join(", "))
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
fn two_unconnected_groups_are_flagged_unless_one_is_short() {
    let even = Sources::file(ONE, Text::new(&format!("{}{}", Items::chain("A", GROUP), Items::chain("B", GROUP))));
    let uneven =
        Sources::file(ONE, Text::new(&format!("{}{}", Items::chain("A", GROUP), Items::chain("B", SHORT_GROUP))));
    assert_eq!(Items::flagged(&[even]), FLAGGED);
    assert_eq!(Items::flagged(&[uneven]), CLEAN);
}

#[test]
fn loosely_linked_groups_are_flagged_and_tightly_linked_ones_are_not() {
    let linked = format!("{}{}pub struct Bridge(A0, B0);\n", Items::clique("A", CLIQUE), Items::clique("B", CLIQUE));
    let all = "A0, A1, A2, A3, A4, A5, B0, B1, B2, B3, B4, B5";
    let tight = format!("{linked}pub struct Second({all});\npub struct Third({all});\n");
    assert_eq!(Items::flagged(&[Sources::file(ONE, Text::new(&linked))]), FLAGGED);
    assert_eq!(Items::flagged(&[Sources::file(ONE, Text::new(&Items::clique("A", CLIQUE)))]), CLEAN);
    assert_eq!(Items::flagged(&[Sources::file(ONE, Text::new(&tight))]), CLEAN);
}

#[test]
fn a_crate_is_judged_on_all_its_files_together() {
    let first = Sources::file(ONE, Text::new(&format!("mod other;\n{}", Items::chain("A", GROUP))));
    let second = Sources::file("crates/one/src/other.rs", Text::new(&Items::chain("B", GROUP)));
    assert_eq!(Items::flagged(&[first, second]), FLAGGED);
}

#[test]
fn crates_are_judged_apart() {
    let one = Sources::file(ONE, Text::new(&Items::chain("A", GROUP)));
    let two = Sources::file(TWO, Text::new(&Items::chain("B", GROUP)));
    assert_eq!(Items::flagged(&[one, two]), CLEAN);
}

#[test]
fn each_flagged_crate_is_named_once_per_finding() {
    let one = Sources::file(ONE, Text::new(&format!("{}{}", Items::chain("A", GROUP), Items::chain("B", GROUP))));
    let two = Sources::file(TWO, Text::new(&format!("{}{}", Items::chain("A", GROUP), Items::chain("B", GROUP))));
    assert_eq!(Items::flagged(&[one, two]), ["crates/one/src/lib.rs:1", "crates/two/src/lib.rs:1"]);
}

#[test]
fn test_code_does_not_count() {
    let in_module = Sources::file(
        ONE,
        Text::new(&format!("{}#[cfg(test)]\nmod tests {{\n{}}}\n", Items::chain("A", GROUP), Items::chain("B", GROUP))),
    );
    let in_target = Sources::file("crates/one/tests/flow.rs", Text::new(&Items::chain("B", GROUP)));
    let shipped = Sources::file(ONE, Text::new(&Items::chain("A", GROUP)));
    assert_eq!(Items::flagged(&[in_module]), CLEAN);
    assert_eq!(Items::flagged(&[shipped, in_target]), CLEAN);
}

const JOIN: &str = "crates/one/src/join.rs";
const JOINED: &str = "pub struct Join(A0, A1, A2, A3, A4, B0, B1, B2, B3, B4);\n";
const ELSEWHERE: &str = "use elsewhere::{A0, A1, A2, A3, A4};\n";

impl Items {
    fn joined(imports: &str, join: &str) -> Lines {
        let groups = format!("mod join;\n{}{}", Items::chain("A", GROUP), Items::chain("B", GROUP));
        Items::flagged(&[
            Sources::file(ONE, Text::new(&groups)),
            Sources::file(JOIN, Text::new(&format!("{imports}{join}"))),
        ])
    }
}

#[test]
fn a_name_from_another_crate_is_not_a_link() {
    assert_eq!(Items::joined("", JOINED), CLEAN);
    assert_eq!(Items::joined(ELSEWHERE, JOINED), FLAGGED);
}

#[test]
fn items_are_judged_whatever_else_the_crate_holds() {
    let loose = format!("{}{}pub struct Bridge(A0, B0);\n", Items::clique("A", CLIQUE), Items::clique("B", CLIQUE));
    let unconnected = format!("{}{}", Items::chain("A", GROUP), Items::chain("B", GROUP));
    let tight_group = format!("{}pub struct C3(A5);\n{}", Items::clique("C", SHORT_GROUP - 1), "");
    let stray = "pub struct Stray;\n";
    assert_eq!(Items::flagged(&[Sources::file(ONE, Text::new(&format!("{loose}{stray}")))]), FLAGGED);
    assert_eq!(Items::flagged(&[Sources::file(ONE, Text::new(&format!("{unconnected}{stray}")))]), FLAGGED);
    assert_eq!(Items::flagged(&[Sources::file(ONE, Text::new(&format!("{loose}{tight_group}")))]), FLAGGED);
}

#[test]
fn unrelated_items_are_not_a_group() {
    let lines: Vec<String> = (0..GROUP).map(|index| format!("pub struct S{index};\n")).collect();
    let strays = lines.concat();
    let crowd = format!("{}{strays}", Items::clique("A", 2 * GROUP));
    assert_eq!(Items::flagged(&[Sources::file(ONE, Text::new(&crowd))]), CLEAN);
}

#[test]
fn a_long_chain_is_judged_without_exhausting_the_stack_and_is_loose() {
    assert_eq!(Items::flagged(&[Sources::file(ONE, Text::new(&Items::chain("A", LONG_CHAIN)))]), FLAGGED);
}

#[test]
fn a_renamed_import_and_a_module_path_both_link() {
    let renamed = "use crate::a::{A0 as P0, A1 as P1, A2 as P2, A3 as P3, A4 as P4};\n";
    let by_path = "use crate::a;\n";
    let flagged = |header: &str, names: &str| {
        let lib = format!("mod a;\nmod b;\n{}", Items::chain("B", GROUP));
        let join = format!("{header}pub struct Join({names}, B0, B1, B2, B3, B4);\n");
        Items::flagged(&[
            Sources::file(ONE, Text::new(&lib)),
            Sources::file("crates/one/src/a.rs", Text::new(&Items::chain("A", GROUP))),
            Sources::file("crates/one/src/b.rs", Text::new(&join)),
        ])
    };
    assert_eq!(flagged(renamed, "P0, P1, P2, P3, P4"), CLEAN);
    assert_eq!(flagged(by_path, "a::A0, a::A1, a::A2, a::A3, a::A4"), CLEAN);
    assert_eq!(flagged("use crate::a::{self};\n", "a::A0, a::A1, a::A2, a::A3, a::A4"), CLEAN);
}

#[test]
fn an_import_in_test_code_does_not_hide_links_in_shipped_code() {
    let groups = format!("{}{}{}", Items::chain("A", GROUP), Items::chain("B", GROUP), JOINED);
    let tests = "#[cfg(test)]\nmod tests {\n    use other::{A0, A1, A2, A3, A4};\n}\n";
    assert_eq!(Items::flagged(&[Sources::file(ONE, Text::new(&format!("{groups}{tests}")))]), CLEAN);
}

#[test]
fn items_the_crate_does_not_declare_are_not_counted() {
    let groups = format!("{}{}", Items::chain("A", GROUP), Items::chain("B", SHORT_GROUP));
    let foreign = "impl From<B0> for String {\n    fn from(_: B0) -> String {\n        String::new()\n    }\n}\n";
    let unnamed = "const _: B0 = B0(B1(B2(B3)));\n";
    assert_eq!(Items::flagged(&[Sources::file(ONE, Text::new(&format!("{groups}{foreign}{unnamed}")))]), CLEAN);
}

#[test]
fn items_declared_by_a_macro_call_are_judged() {
    let groups = format!("{}{}", Items::chain("A", GROUP), Items::chain("B", GROUP));
    let wrapped = format!("macro_rules! wrap {{ ($($item:item)*) => {{ $($item)* }} }}\nwrap! {{\n{groups}}}\n");
    assert_eq!(Items::flagged(&[Sources::file(ONE, Text::new(&wrapped))]), FLAGGED);
}

const IMPORTS: &str = "use elsewhere::{A0, A1, A2, A3, A4};";

#[test]
fn imports_inside_items_still_hide_links() {
    let holders = [
        format!("pub struct Holder;\nimpl Holder {{\n    fn hold() {{\n        {IMPORTS}\n    }}\n}}\n"),
        format!("pub fn hold() {{\n    {IMPORTS}\n}}\n"),
        format!("mod holder {{\n    {IMPORTS}\n}}\n"),
    ];
    assert_eq!(holders.map(|holder| Items::joined(&holder, JOINED)), [FLAGGED; 3]);
}

#[test]
fn a_path_rooted_elsewhere_is_not_a_link() {
    let join = "pub struct Join(other::A0, other::A1, other::A2, other::A3, other::A4, B0, B1, B2, B3, B4);\n";
    assert_eq!(Items::joined("", join), FLAGGED);
}

#[test]
fn every_kind_of_item_is_a_node() {
    let kinds = "pub enum A0 { Next(A1) }\npub trait A1: A2 {}\npub fn A2(next: A3) {}\npub type A3 = A4;\npub const A4: u8 = 1;\n";
    let statics =
        "pub static A0: A1 = A1(A2);\npub struct A1(A2);\npub struct A2(A3);\npub struct A3(A4);\npub struct A4;\n";
    let moduled = format!("mod inner {{\n{}}}\n", Items::chain("A", GROUP));
    let groups = [kinds, statics, &moduled].map(|group| format!("{group}{}", Items::chain("B", GROUP)));
    let flagged = groups.map(|text| Items::flagged(&[Sources::file(ONE, Text::new(&text))]));
    assert_eq!(flagged, [FLAGGED; 3]);
}

#[test]
fn links_inside_impls_count_unless_the_impl_is_test_code() {
    let groups = format!("{}{}pub struct Join;\n", Items::chain("A", GROUP), Items::chain("B", GROUP));
    let join = "impl Join {\n    fn take(items: (A0, A1, A2, A3, A4), others: (B0, B1, B2, B3, B4)) {}\n}\n";
    assert_eq!(Items::flagged(&[Sources::file(ONE, Text::new(&format!("{groups}{join}")))]), CLEAN);
    assert_eq!(Items::flagged(&[Sources::file(ONE, Text::new(&format!("{groups}#[cfg(test)]\n{join}")))]), FLAGGED);
}
