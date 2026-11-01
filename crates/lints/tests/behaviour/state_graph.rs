use lints::StateGraph;

use crate::common::{Case, Source, Sources};

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
    Case { name: "exposed field", sources: &[Source::new("a.rs", EXPOSED)], flagged: &["a.rs:2"] },
    Case { name: "foreign match", sources: &[SHAPE, MATCHING], flagged: &["area.rs:3"] },
    Case { name: "own match", sources: &[SHAPE], flagged: &[] },
    Case {
        name: "hidden edge",
        sources: &[Source::new("a.rs", OPEN_AND_CLOSED), Source::new("b.rs", HIDDEN_EDGE)],
        flagged: &["b.rs:2"],
    },
    Case {
        name: "mutating a vertex",
        sources: &[Source::new("a.rs", OPEN_AND_CLOSED), Source::new("b.rs", MUTATES)],
        flagged: &["b.rs:2"],
    },
    Case {
        name: "public edge",
        sources: &[Source::new("a.rs", OPEN_AND_CLOSED), Source::new("b.rs", EDGE)],
        flagged: &[],
    },
    Case { name: "test module match", sources: &[SHAPE, TEST_AREA], flagged: &[] },
    Case { name: "own unit enum of that name", sources: &[SHAPE, UNIT_SHAPE], flagged: &[] },
];

#[test]
fn state_graph_rules_are_flagged() {
    assert_eq!(Sources::mismatches::<StateGraph>(&CASES), Vec::<String>::new());
}
