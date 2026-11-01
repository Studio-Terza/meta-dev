use lint_no_parameter_bags::NoParameterBags;

use lint_fixtures::{Case, Source, Sources};

const BAG: Source = Source::new("bag.rs", "pub struct Options {\n    pub depth: u8,\n}\n");
const WITH_LOGIC: Source = Source::new(
    "bag.rs",
    "pub struct Options {\n    depth: u8,\n}\nimpl Options {\n    pub fn deeper(&self) -> u8 {\n        self.depth + 1\n    }\n}\n",
);
const FIRST_USER: Source = Source::new("first.rs", "fn run(options: Options) {}\n");
const SECOND_USER: Source = Source::new("second.rs", "fn walk(options: Options) {}\n");

const PRIVATE_BAG: Source = Source::new("crates/a/src/bag.rs", "struct Options {\n    depth: u8,\n}\n");
const EXPORTED_BAG: Source = Source::new("crates/a/src/bag.rs", "pub struct Options {\n    pub depth: u8,\n}\n");
const ROOT: Source = Source::new("crates/a/src/lib.rs", "mod bag;\npub use bag::Options;\n");
const INNER_USER: Source = Source::new("crates/a/src/first.rs", "fn run(options: Options) {}\n");
const OTHER_CRATE_USER: Source = Source::new("crates/b/src/lib.rs", "fn run(options: Options) {}\n");

const CASES: [Case; 7] = [
    Case::new("one consumer", &[BAG, FIRST_USER], &["bag.rs:1"]),
    Case::new("two consumers", &[BAG, FIRST_USER, SECOND_USER], &[]),
    Case::new("own logic", &[WITH_LOGIC, FIRST_USER], &[]),
    Case::new("another crate's namesake is no consumer", &[PRIVATE_BAG, INNER_USER, OTHER_CRATE_USER], &["crates/a/src/bag.rs:1"]),
    Case::new("exported from the library root", &[EXPORTED_BAG, ROOT], &[]),
    Case::new("exported and used elsewhere", &[EXPORTED_BAG, ROOT, INNER_USER, OTHER_CRATE_USER], &[]),
    Case::new("no consumer", &[BAG], &[]),
];

#[test]
fn structs_without_logic_that_one_file_consumes_are_flagged() {
    assert_eq!(Sources::mismatches::<NoParameterBags>(&CASES), Vec::<String>::new());
}
