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

const NONE: [&str; 0] = [];
const SOUND: &str = "states! {\n    pub mod pr {\n        start Working { review -> Reviewing, abandon -> Abandoned }\n        state Reviewing { approve -> Merged, rework -> Working }\n        terminal Merged {}\n        terminal Abandoned {}\n    }\n}\n";
const NESTED: &str = "fn f() {\n    mod inner {\n        states::states! {\n            #[derive()]\n            mod door {\n                start Open { shut -> Shut, }\n                terminal Shut {}\n                state Lost {}\n            }\n        }\n    }\n}\n";
const UNREACHABLE: &str = "states! {\n    mod door {\n        start Open { shut -> Shut }\n        state Ajar { shut -> Shut }\n        terminal Shut {}\n    }\n}\n";
const LOOPING: &str = "states! {\n    mod door {\n        start Open { swing -> Ajar, shut -> Shut }\n        state Ajar { stick -> Stuck }\n        state Stuck { stick -> Ajar }\n        terminal Shut {}\n    }\n}\n";
const OPEN_TERMINAL: &str = "states! {\n    mod door {\n        start Open { shut -> Shut }\n        terminal Shut { reopen -> Open }\n    }\n}\n";
const NO_START: &str =
    "states! {\n    mod door {\n        state Open { shut -> Shut }\n        terminal Shut {}\n    }\n}\n";
const NO_TERMINAL: &str = "states! {\n    mod door {\n        start Open {}\n    }\n}\n";
const UNKNOWN_KIND: &str = "states! {\n    mod door {\n        begin Open {}\n    }\n}\n";
const NOT_A_GRAPH: &str = "fn f() {\n    states!(1 + 1);\n}\n";
const IN_TEST_MODULE: &str =
    "#[cfg(test)]\nmod tests {\n    states! {\n        mod door {\n            begin Open {}\n        }\n    }\n}\n";
const OTHER_MACRO: &str = "fn f() {\n    print!(\"start Open\");\n}\n";
const THE_DEFINITION: &str = "macro_rules! states {\n    () => {};\n}\n";

const ALIASED: &str = "use states::states as lifecycle;\nlifecycle! {\n    mod door {\n        start Open { shut -> Shut }\n        state Lost {}\n        terminal Shut {}\n    }\n}\n";
const GROUPED_ALIAS: &str = "use states::{states as lifecycle, other};\nfn f() {\n    lifecycle! {\n        mod door {\n            start Open { shut -> Shut }\n            state Lost {}\n            terminal Shut {}\n        }\n    }\n}\n";
const BEHIND_CFG_IF: &str = "cfg_if::cfg_if! {\n    if #[cfg(unix)] {\n        states::states! {\n            mod door {\n                start Open { shut -> Shut }\n                state Lost {}\n                terminal Shut {}\n            }\n        }\n    }\n}\n";
const BEHIND_MACRO_RULES: &str = "macro_rules! door {\n    () => {\n        states::states! {\n            mod door {\n                start Open { shut -> Shut }\n                state Lost {}\n                terminal Shut {}\n            }\n        }\n    };\n}\ndoor!();\n";

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

laws::table!(declared_graphs_that_hold_pass, Sources::flagged::<StateGraph>, [
    [Source::new("a.rs", SOUND)] => NONE,
    [Source::new("a.rs", IN_TEST_MODULE)] => NONE,
    [Source::new("x/tests/a.rs", UNREACHABLE)] => NONE,
    [Source::new("a.rs", OTHER_MACRO)] => NONE,
    [Source::new("a.rs", THE_DEFINITION)] => NONE,
]);

laws::table!(declared_graphs_that_break_are_flagged, Sources::flagged::<StateGraph>, [
    [Source::new("a.rs", NESTED)] => ["a.rs:8", "a.rs:8"],
    [Source::new("a.rs", UNREACHABLE)] => ["a.rs:4"],
    [Source::new("a.rs", LOOPING)] => ["a.rs:4", "a.rs:5"],
    [Source::new("a.rs", OPEN_TERMINAL)] => ["a.rs:4"],
    [Source::new("a.rs", NO_START)] => ["a.rs:3", "a.rs:4"],
    [Source::new("a.rs", NO_TERMINAL)] => ["a.rs:3"],
    [Source::new("a.rs", UNKNOWN_KIND)] => ["a.rs:1"],
    [Source::new("a.rs", NOT_A_GRAPH)] => ["a.rs:2"],
    [Source::new("a.rs", ALIASED)] => ["a.rs:5", "a.rs:5"],
    [Source::new("a.rs", GROUPED_ALIAS)] => ["a.rs:6", "a.rs:6"],
    [Source::new("a.rs", BEHIND_CFG_IF)] => ["a.rs:6", "a.rs:6"],
    [Source::new("a.rs", BEHIND_MACRO_RULES)] => ["a.rs:6", "a.rs:6"],
]);

#[test]
fn the_messages_name_the_state() {
    let shown: String = [UNREACHABLE, LOOPING, OPEN_TERMINAL, UNKNOWN_KIND]
        .iter()
        .map(|text| Sources::report::<StateGraph>(&[Source::new("a.rs", text)]).to_string())
        .collect();
    let named = [
        "state Ajar cannot be reached",
        "state Stuck cannot reach a terminal",
        "terminal state Shut has outgoing",
        "graph cannot be read",
    ];
    assert!(named.iter().all(|word| shown.contains(word)));
}
