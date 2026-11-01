use lint_state_graph::StateGraph;

use lint_fixtures::{Case, Source, Sources};

const EXPOSED: &str = "pub struct A {\n    pub depth: u8,\n    hidden: u8,\n}\n";
const OPEN_AND_CLOSED: &str = "struct Open;\nstruct Closed;\nimpl State for Open {}\nimpl State for Closed {}\n";
const EDGE: &str = "impl Open {\n    pub fn close(self) -> Closed {\n        Closed\n    }\n}\n";
const HIDDEN_EDGE: &str = "impl Open {\n    fn close(self) -> Closed {\n        Closed\n    }\n}\n";
const MUTATES: &str = "impl Open {\n    pub fn tweak(&mut self) {}\n}\n";
const SHAPE: Source = Source::new("shape.rs", "pub enum Shape {\n    Circle(u8),\n    Square,\n}\n");
const MATCHING: Source = Source::new(
    "area.rs",
    "fn area(shape: Shape) -> u8 {\n    match shape {\n        Shape::Circle(radius) => grow(radius),\n        Shape::Square => 1,\n    }\n}\n",
);

const TEST_AREA: Source = Source::new(
    "area.rs",
    "#[cfg(test)]\nmod tests {\n    fn area(shape: Shape) -> u8 {\n        match shape {\n            Shape::Circle(radius) => grow(radius),\n            Shape::Square => 1,\n        }\n    }\n}\n",
);
const UNIT_SHAPE: Source = Source::new(
    "unit.rs",
    "enum Shape {\n    Circle,\n    Square,\n}\nfn area(shape: Shape) -> u8 {\n    match shape {\n        Shape::Circle => grow(1),\n        Shape::Square => grow(2),\n    }\n}\n",
);

const CASES: [Case; 8] = [
    Case::new("exposed field", &[Source::new("a.rs", EXPOSED)], &["a.rs:2"]),
    Case::new("foreign match", &[SHAPE, MATCHING], &["area.rs:3"]),
    Case::new("own match", &[SHAPE], &[]),
    Case::new("hidden edge", &[Source::new("a.rs", OPEN_AND_CLOSED), Source::new("b.rs", HIDDEN_EDGE)], &["b.rs:2"]),
    Case::new("mutating a vertex", &[Source::new("a.rs", OPEN_AND_CLOSED), Source::new("b.rs", MUTATES)], &["b.rs:2"]),
    Case::new("public edge", &[Source::new("a.rs", OPEN_AND_CLOSED), Source::new("b.rs", EDGE)], &[]),
    Case::new("test module match", &[SHAPE, TEST_AREA], &[]),
    Case::new("own unit enum of that name", &[SHAPE, UNIT_SHAPE], &[]),
];

#[test]
fn state_graph_rules_are_flagged() {
    assert_eq!(Sources::mismatches::<StateGraph>(&CASES), Vec::<String>::new());
}
