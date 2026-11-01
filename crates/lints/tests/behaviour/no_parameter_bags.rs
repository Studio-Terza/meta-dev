use lints::NoParameterBags;

use crate::common::{Case, Source, Sources};

const BAG: Source = Source::new("bag.rs", "pub struct Options {\n    pub depth: u8,\n}\n");
const WITH_LOGIC: Source = Source::new(
    "bag.rs",
    "pub struct Options {\n    depth: u8,\n}\nimpl Options {\n    pub fn deeper(&self) -> u8 {\n        self.depth + 1\n    }\n}\n",
);
const FIRST_USER: Source = Source::new("first.rs", "fn run(options: Options) {}\n");
const SECOND_USER: Source = Source::new("second.rs", "fn walk(options: Options) {}\n");

const CASES: [Case; 4] = [
    Case { name: "one consumer", sources: &[BAG, FIRST_USER], flagged: &["bag.rs:1"] },
    Case { name: "two consumers", sources: &[BAG, FIRST_USER, SECOND_USER], flagged: &[] },
    Case { name: "own logic", sources: &[WITH_LOGIC, FIRST_USER], flagged: &[] },
    Case { name: "no consumer", sources: &[BAG], flagged: &[] },
];

#[test]
fn structs_without_logic_that_one_file_consumes_are_flagged() {
    assert_eq!(Sources::mismatches::<NoParameterBags>(&CASES), Vec::<String>::new());
}
