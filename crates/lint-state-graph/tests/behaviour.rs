use lint_state_graph::StateGraph;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

const OPEN_AND_CLOSED: &str = "struct Open;\nstruct Closed;\nimpl State for Open {}\nimpl State for Closed {}\n";
const EDGE: &str = "impl Open {\n    pub fn close(self) -> Closed {\n        Closed\n    }\n}\n";
const MUTATES: &str = "impl Open {\n    pub fn tweak(&mut self) {}\n}\n";
const SHAPE: Source =
    Source::new(Located::new("shape.rs"), Text::new("pub enum Shape {\n    Circle(u8),\n    Square,\n}\n"));

const TEST_AREA: Source = Source::new(
    Located::new("area.rs"),
    Text::new(
        "#[cfg(test)]\nmod tests {\n    fn area(shape: Shape) -> u8 {\n        match shape {\n            Shape::Circle(radius) => grow(radius),\n            Shape::Square => 1,\n        }\n    }\n}\n",
    ),
);
const UNIT_SHAPE: Source = Source::new(
    Located::new("unit.rs"),
    Text::new(
        "enum Shape {\n    Circle,\n    Square,\n}\nfn area(shape: Shape) -> u8 {\n    match shape {\n        Shape::Circle => grow(1),\n        Shape::Square => grow(2),\n    }\n}\n",
    ),
);

const NONE: [&str; 0] = [];
const NESTED: &str = "fn f() {\n    mod inner {\n        states::states! {\n            #[derive()]\n            mod door {\n                start Open { shut -> Shut, }\n                terminal Shut {}\n                state Lost {}\n            }\n        }\n    }\n}\n";
const UNREACHABLE: &str = "states! {\n    mod door {\n        start Open { shut -> Shut }\n        state Ajar { shut -> Shut }\n        terminal Shut {}\n    }\n}\n";
const NO_START: &str =
    "states! {\n    mod door {\n        state Open { shut -> Shut }\n        terminal Shut {}\n    }\n}\n";
const NO_TERMINAL: &str = "states! {\n    mod door {\n        start Open {}\n    }\n}\n";
const OTHER_RENAME: &str = "use other::thing as lifecycle;\nlifecycle! {\n    mod door {\n        start Open { shut -> Shut }\n        state Lost {}\n        terminal Shut {}\n    }\n}\n";
const NOT_A_GRAPH: &str = "fn f() {\n    states!(1 + 1);\n}\n";
const IN_TEST_MODULE: &str =
    "#[cfg(test)]\nmod tests {\n    states! {\n        mod door {\n            begin Open {}\n        }\n    }\n}\n";
const OTHER_MACRO: &str = "fn f() {\n    print!(\"start Open\");\n}\n";
const THE_DEFINITION: &str = "macro_rules! states {\n    () => {};\n}\n";

const VALUE_VIEW: &str = "impl Value {\n    pub fn report(&self) -> Report {\n        make()\n    }\n}\n";
const SELF_VIEW: &str = "impl Open {\n    pub fn copy(&self) -> Self {\n        make()\n    }\n}\n";
const OPTION_VIEW: &str = "impl Open {\n    pub fn next(&self) -> Option<Closed> {\n        make()\n    }\n}\n";
const FIELD_VIEW: &str = "impl Open {\n    pub fn next(&self) -> Closed {\n        self.closed\n    }\n}\n";
const OTHER_FIELD_VIEW: &str =
    "impl Open {\n    pub fn next(&self, other: Other) -> Closed {\n        other.closed\n    }\n}\n";
const ALIAS_VERTEX: &str = "type Door = Open;\nimpl State for Door {}\n";
const INLINE_VERTEX: &str = "mod inner {\n    pub struct Open;\n    impl State for Open {}\n    impl Open {\n        pub fn tweak(&mut self) {}\n    }\n}\n";
const INLINE_FIELD: &str = "mod inner {\n    pub struct A {\n        pub depth: u8,\n    }\n}\n";
const FREE_TRANSITION: &str =
    "struct Maker;\nimpl Maker {\n    pub fn close(open: Open) -> Closed {\n        make()\n    }\n}\n";
const TRAIT_TRANSITION: &str =
    "impl From<Open> for Closed {\n    fn from(open: Open) -> Closed {\n        make()\n    }\n}\n";
const TWO_TRAITS: &str = "impl Wrap for Open {\n    fn wrap(self) -> Closed {\n        make()\n    }\n}\nimpl Seal for Open {\n    fn seal(self) -> Closed {\n        make()\n    }\n}\n";

const INLINE_SHAPE: Source = Source::new(
    Located::new("shape.rs"),
    Text::new("mod inner {\n    pub enum Shape {\n        Circle(u8),\n        Square,\n    }\n}\n"),
);
const FOREIGN_MATCH: Source = Source::new(
    Located::new("area.rs"),
    Text::new(
        "fn area(shape: Shape) -> u8 {\n    match shape {\n        Shape::Circle(radius) => grow(radius),\n        Shape::Square => 1,\n    }\n}\n",
    ),
);
const COLOR: Source = Source::new(Located::new("color.rs"), Text::new("pub enum Color {\n    Red,\n    Blue,\n}\n"));
const PAINT: Source = Source::new(
    Located::new("paint.rs"),
    Text::new(
        "fn paint(color: Color) -> u8 {\n    match color {\n        Color::Red => grow(1),\n        Color::Blue => grow(2),\n    }\n}\n",
    ),
);
const SUM_SHAPE_X: Source = Source::new(
    Located::new("crates/x/src/shape.rs"),
    Text::new("pub enum Shape {\n    Circle(u8),\n    Square,\n}\n"),
);
const UNIT_SHAPE_X: Source =
    Source::new(Located::new("crates/x/src/shape.rs"), Text::new("pub enum Shape {\n    Circle,\n    Square,\n}\n"));
const SUM_SHAPE_Y: Source = Source::new(
    Located::new("crates/y/src/shape.rs"),
    Text::new("pub enum Shape {\n    Circle(u8),\n    Square,\n}\n"),
);
const UNIT_SHAPE_Y: Source =
    Source::new(Located::new("crates/y/src/shape.rs"), Text::new("pub enum Shape {\n    Circle,\n    Square,\n}\n"));
const AREA_X: Source = Source::new(
    Located::new("crates/x/src/area.rs"),
    Text::new(
        "fn area(shape: Shape) -> u8 {\n    match shape {\n        Shape::Circle(radius) => grow(radius),\n        Shape::Square => grow(1),\n    }\n}\n",
    ),
);
const UNIT_AREA_X: Source = Source::new(
    Located::new("crates/x/src/area.rs"),
    Text::new(
        "fn area(shape: Shape) -> u8 {\n    match shape {\n        Shape::Circle => grow(1),\n        Shape::Square => grow(2),\n    }\n}\n",
    ),
);
const BOXED_VIEW: &str = "impl Open {\n    pub fn next(&self) -> Box<Closed> {\n        make()\n    }\n}\n";
const UNIT_ARM: Source = Source::new(
    Located::new("area.rs"),
    Text::new(
        "fn area(shape: Shape) -> u8 {\n    match shape {\n        Shape::Square => grow(1),\n        _ => 2,\n    }\n}\n",
    ),
);
const ROUND: Source =
    Source::new(Located::new("shape.rs"), Text::new("pub enum Shape {\n    Round { radius: u8 },\n}\n"));
const STRUCT_ARM: Source = Source::new(
    Located::new("area.rs"),
    Text::new(
        "fn area(shape: Shape) -> u8 {\n    match shape {\n        Shape::Round { radius } => grow(radius),\n    }\n}\n",
    ),
);
const WIDE_SHAPE: Source = Source::new(
    Located::new("shape.rs"),
    Text::new("pub enum Shape {\n    Circle(u8),\n    Square,\n    Oval(u8),\n    Star(u8),\n}\n"),
);
const TABLES: Source = Source::new(
    Located::new("area.rs"),
    Text::new(
        "fn area(shape: Shape) -> Row {\n    match shape {\n        Shape::Circle(_) => -1,\n        Shape::Square => (2),\n        Shape::Oval(_) => (1, 2),\n        Shape::Star(_) => [1, 2],\n    }\n}\n",
    ),
);
const IN_METHOD: Source = Source::new(
    Located::new("area.rs"),
    Text::new(
        "impl Area {\n    fn area(shape: Shape) -> u8 {\n        match shape {\n            Shape::Circle(radius) => grow(radius),\n            Shape::Square => 1,\n        }\n    }\n}\n",
    ),
);
const IN_MODULE: Source = Source::new(
    Located::new("area.rs"),
    Text::new(
        "mod inner {\n    fn area(shape: Shape) -> u8 {\n        match shape {\n            Shape::Circle(radius) => grow(radius),\n            Shape::Square => 1,\n        }\n    }\n}\n",
    ),
);
const IF_LET: Source = Source::new(
    Located::new("area.rs"),
    Text::new(
        "fn area(shape: Shape) -> u8 {\n    if let Shape::Circle(radius) = shape { grow(radius) } else { 1 }\n}\n",
    ),
);
const LET_ELSE: Source = Source::new(
    Located::new("area.rs"),
    Text::new(
        "fn area(shape: Shape) -> u8 {\n    let Shape::Circle(radius) = shape else { panic() };\n    grow(radius)\n}\n",
    ),
);

const ALIASED: &str = "use states::states as lifecycle;\nlifecycle! {\n    mod door {\n        start Open { shut -> Shut }\n        state Lost {}\n        terminal Shut {}\n    }\n}\n";
const GROUPED_ALIAS: &str = "use states::{states as lifecycle, other};\nfn f() {\n    lifecycle! {\n        mod door {\n            start Open { shut -> Shut }\n            state Lost {}\n            terminal Shut {}\n        }\n    }\n}\n";
const BEHIND_CFG_IF: &str = "cfg_if::cfg_if! {\n    if #[cfg(unix)] {\n        states::states! {\n            mod door {\n                start Open { shut -> Shut }\n                state Lost {}\n                terminal Shut {}\n            }\n        }\n    }\n}\n";
const BEHIND_MACRO_RULES: &str = "macro_rules! door {\n    () => {\n        states::states! {\n            mod door {\n                start Open { shut -> Shut }\n                state Lost {}\n                terminal Shut {}\n            }\n        }\n    };\n}\ndoor!();\n";

const CASES: [Case; 28] = [
    Case::new(
        Title::new("public edge"),
        &[
            Source::new(Located::new("a.rs"), Text::new(OPEN_AND_CLOSED)),
            Source::new(Located::new("b.rs"), Text::new(EDGE)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(Title::new("test module match"), &[SHAPE, TEST_AREA], Flagged::new(&[])),
    Case::new(Title::new("own unit enum of that name"), &[SHAPE, UNIT_SHAPE], Flagged::new(&[])),
    Case::new(
        Title::new("a value's view"),
        &[
            Source::new(Located::new("a.rs"), Text::new(OPEN_AND_CLOSED)),
            Source::new(Located::new("b.rs"), Text::new(VALUE_VIEW)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a view returning Self"),
        &[
            Source::new(Located::new("a.rs"), Text::new(OPEN_AND_CLOSED)),
            Source::new(Located::new("b.rs"), Text::new(SELF_VIEW)),
        ],
        Flagged::new(&["b.rs:2"]),
    ),
    Case::new(
        Title::new("a view returning an option"),
        &[
            Source::new(Located::new("a.rs"), Text::new(OPEN_AND_CLOSED)),
            Source::new(Located::new("b.rs"), Text::new(OPTION_VIEW)),
        ],
        Flagged::new(&["b.rs:2"]),
    ),
    Case::new(
        Title::new("a view of a field"),
        &[
            Source::new(Located::new("a.rs"), Text::new(OPEN_AND_CLOSED)),
            Source::new(Located::new("b.rs"), Text::new(FIELD_VIEW)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a view of another value's field"),
        &[
            Source::new(Located::new("a.rs"), Text::new(OPEN_AND_CLOSED)),
            Source::new(Located::new("b.rs"), Text::new(OTHER_FIELD_VIEW)),
        ],
        Flagged::new(&["b.rs:2"]),
    ),
    Case::new(
        Title::new("a vertex behind an alias"),
        &[
            Source::new(Located::new("a.rs"), Text::new(ALIAS_VERTEX)),
            Source::new(Located::new("b.rs"), Text::new(MUTATES)),
        ],
        Flagged::new(&["b.rs:2"]),
    ),
    Case::new(
        Title::new("a vertex in an inline module"),
        &[Source::new(Located::new("a.rs"), Text::new(INLINE_VERTEX))],
        Flagged::new(&["a.rs:5"]),
    ),
    Case::new(
        Title::new("a field in an inline module"),
        &[Source::new(Located::new("a.rs"), Text::new(INLINE_FIELD))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("a free transition"),
        &[
            Source::new(Located::new("a.rs"), Text::new(OPEN_AND_CLOSED)),
            Source::new(Located::new("b.rs"), Text::new(FREE_TRANSITION)),
        ],
        Flagged::new(&["b.rs:3"]),
    ),
    Case::new(
        Title::new("a trait transition"),
        &[
            Source::new(Located::new("a.rs"), Text::new(OPEN_AND_CLOSED)),
            Source::new(Located::new("b.rs"), Text::new(TRAIT_TRANSITION)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("one edge through two traits"),
        &[
            Source::new(Located::new("a.rs"), Text::new(OPEN_AND_CLOSED)),
            Source::new(Located::new("b.rs"), Text::new(TWO_TRAITS)),
        ],
        Flagged::new(&["b.rs:2"]),
    ),
    Case::new(Title::new("an enum in an inline module"), &[INLINE_SHAPE, FOREIGN_MATCH], Flagged::new(&["area.rs:3"])),
    Case::new(Title::new("a unit enum elsewhere"), &[COLOR, PAINT], Flagged::new(&[])),
    Case::new(Title::new("arms that only tabulate"), &[WIDE_SHAPE, TABLES], Flagged::new(&[])),
    Case::new(Title::new("a match in a method"), &[SHAPE, IN_METHOD], Flagged::new(&["area.rs:4"])),
    Case::new(Title::new("a match in an inline module"), &[SHAPE, IN_MODULE], Flagged::new(&["area.rs:4"])),
    Case::new(Title::new("if let"), &[SHAPE, IF_LET], Flagged::new(&["area.rs:2"])),
    Case::new(Title::new("let else"), &[SHAPE, LET_ELSE], Flagged::new(&["area.rs:2"])),
    Case::new(
        Title::new("a view returning another wrapper"),
        &[
            Source::new(Located::new("a.rs"), Text::new(OPEN_AND_CLOSED)),
            Source::new(Located::new("b.rs"), Text::new(BOXED_VIEW)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(Title::new("a unit variant arm"), &[SHAPE, UNIT_ARM], Flagged::new(&["area.rs:3"])),
    Case::new(Title::new("a struct variant arm"), &[ROUND, STRUCT_ARM], Flagged::new(&["area.rs:3"])),
    Case::new(
        Title::new("a unit enum of the crate's own beside a sum type of that name in another crate"),
        &[SUM_SHAPE_Y, UNIT_SHAPE_X, UNIT_AREA_X],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a sum type of the crate's own beside a unit enum of that name in another crate"),
        &[UNIT_SHAPE_Y, SUM_SHAPE_X, AREA_X],
        Flagged::new(&["crates/x/src/area.rs:3", "crates/x/src/area.rs:4"]),
    ),
    Case::new(
        Title::new("a sum type of the file's own beside one of that name in another crate"),
        &[
            SUM_SHAPE_Y,
            Source::new(
                Located::new("crates/x/src/area.rs"),
                Text::new(
                    "enum Shape {\n    Circle(u8),\n}\nfn area(shape: Shape) -> u8 {\n    match shape {\n        Shape::Circle(radius) => grow(radius),\n    }\n}\n",
                ),
            ),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a sum type of another file of the crate"),
        &[SUM_SHAPE_X, AREA_X],
        Flagged::new(&["crates/x/src/area.rs:3", "crates/x/src/area.rs:4"]),
    ),
];

#[test]
fn state_graph_rules_are_flagged() {
    assert_eq!(Sources::mismatches::<StateGraph>(&CASES), Lines::default());
}

laws::table!(declared_graphs_that_hold_pass, Sources::flagged::<StateGraph>, [
    [Source::new(Located::new("a.rs"), Text::new(IN_TEST_MODULE))] => NONE,
    [Source::new(Located::new("x/tests/a.rs"), Text::new(UNREACHABLE))] => NONE,
    [Source::new(Located::new("a.rs"), Text::new(OTHER_MACRO))] => NONE,
    [Source::new(Located::new("a.rs"), Text::new(OTHER_RENAME))] => NONE,
    [Source::new(Located::new("a.rs"), Text::new(THE_DEFINITION))] => NONE,
]);

laws::table!(declared_graphs_that_break_are_flagged, Sources::flagged::<StateGraph>, [
    [Source::new(Located::new("a.rs"), Text::new(NESTED))] => ["a.rs:8", "a.rs:8"],
    [Source::new(Located::new("a.rs"), Text::new(NO_START))] => ["a.rs:3", "a.rs:4"],
    [Source::new(Located::new("a.rs"), Text::new(NO_TERMINAL))] => ["a.rs:3"],
    [Source::new(Located::new("a.rs"), Text::new(NOT_A_GRAPH))] => ["a.rs:2"],
    [Source::new(Located::new("a.rs"), Text::new(ALIASED))] => ["a.rs:5", "a.rs:5"],
    [Source::new(Located::new("a.rs"), Text::new(GROUPED_ALIAS))] => ["a.rs:6", "a.rs:6"],
    [Source::new(Located::new("a.rs"), Text::new(BEHIND_CFG_IF))] => ["a.rs:6", "a.rs:6"],
    [Source::new(Located::new("a.rs"), Text::new(BEHIND_MACRO_RULES))] => ["a.rs:6", "a.rs:6"],
]);
