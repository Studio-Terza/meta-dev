use lint_fixtures::{Case, Flagged, Lines, Located, Queried, Source, Sources, Text, Title};
use lint_state_graph::ForeignMatches;

const LIB_X: Source = Source::new(Located::new("crates/x/src/lib.rs"), Text::new("mod area;\nmod shape;\n"));
const LIB_Y: Source = Source::new(Located::new("crates/y/src/lib.rs"), Text::new("mod shape;\n"));
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
        "use crate::shape::Shape;\n\nfn area(shape: Shape) -> u8 {\n    match shape {\n        Shape::Circle(radius) => grow(radius),\n        Shape::Square => grow(1),\n    }\n}\n",
    ),
);
const UNIT_AREA_X: Source = Source::new(
    Located::new("crates/x/src/area.rs"),
    Text::new(
        "use crate::shape::Shape;\n\nfn area(shape: Shape) -> u8 {\n    match shape {\n        Shape::Circle => grow(1),\n        Shape::Square => grow(2),\n    }\n}\n",
    ),
);
const OWN_AREA_X: Source = Source::new(
    Located::new("crates/x/src/area.rs"),
    Text::new(
        "enum Shape {\n    Circle(u8),\n}\n\nfn area(shape: Shape) -> u8 {\n    match shape {\n        Shape::Circle(radius) => grow(radius),\n    }\n}\n",
    ),
);
const TEST_AREA_X: Source = Source::new(
    Located::new("crates/x/src/area.rs"),
    Text::new(
        "#[cfg(test)]\nmod tests {\n    use crate::shape::Shape;\n\n    fn area(shape: Shape) -> u8 {\n        match shape {\n            Shape::Circle(radius) => grow(radius),\n            Shape::Square => grow(1),\n        }\n    }\n}\n",
    ),
);
const INLINE_SHAPE_X: Source = Source::new(
    Located::new("crates/x/src/shape.rs"),
    Text::new("pub mod inner {\n    pub enum Shape {\n        Circle(u8),\n        Square,\n    }\n}\n"),
);
const INLINE_AREA_X: Source = Source::new(
    Located::new("crates/x/src/area.rs"),
    Text::new(
        "use crate::shape::inner::Shape;\n\nfn area(shape: Shape) -> u8 {\n    match shape {\n        Shape::Circle(radius) => grow(radius),\n        Shape::Square => 1,\n    }\n}\n",
    ),
);
const COLOR_X: Source =
    Source::new(Located::new("crates/x/src/shape.rs"), Text::new("pub enum Color {\n    Red,\n    Blue,\n}\n"));
const PAINT_X: Source = Source::new(
    Located::new("crates/x/src/area.rs"),
    Text::new(
        "use crate::shape::Color;\n\nfn paint(color: Color) -> u8 {\n    match color {\n        Color::Red => grow(1),\n        Color::Blue => grow(2),\n    }\n}\n",
    ),
);
const WIDE_SHAPE_X: Source = Source::new(
    Located::new("crates/x/src/shape.rs"),
    Text::new("pub enum Shape {\n    Circle(u8),\n    Square,\n    Oval(u8),\n    Star(u8),\n}\n"),
);
const TABLES_X: Source = Source::new(
    Located::new("crates/x/src/area.rs"),
    Text::new(
        "use crate::shape::Shape;\n\nfn area(shape: Shape) -> Row {\n    match shape {\n        Shape::Circle(_) => -1,\n        Shape::Square => (2),\n        Shape::Oval(_) => (1, 2),\n        Shape::Star(_) => [1, 2],\n    }\n}\n",
    ),
);
const METHOD_X: Source = Source::new(
    Located::new("crates/x/src/area.rs"),
    Text::new(
        "use crate::shape::Shape;\n\nimpl Area {\n    fn area(shape: Shape) -> u8 {\n        match shape {\n            Shape::Circle(radius) => grow(radius),\n            Shape::Square => 1,\n        }\n    }\n}\n",
    ),
);
const MODULE_X: Source = Source::new(
    Located::new("crates/x/src/area.rs"),
    Text::new(
        "mod inner {\n    use crate::shape::Shape;\n\n    fn area(shape: Shape) -> u8 {\n        match shape {\n            Shape::Circle(radius) => grow(radius),\n            Shape::Square => 1,\n        }\n    }\n}\n",
    ),
);
const IF_LET_X: Source = Source::new(
    Located::new("crates/x/src/area.rs"),
    Text::new(
        "use crate::shape::Shape;\n\nfn area(shape: Shape) -> u8 {\n    if let Shape::Circle(radius) = shape { grow(radius) } else { 1 }\n}\n",
    ),
);
const LET_ELSE_X: Source = Source::new(
    Located::new("crates/x/src/area.rs"),
    Text::new(
        "use crate::shape::Shape;\n\nfn area(shape: Shape) -> u8 {\n    let Shape::Circle(radius) = shape else { panic() };\n    grow(radius)\n}\n",
    ),
);
const UNIT_ARM_X: Source = Source::new(
    Located::new("crates/x/src/area.rs"),
    Text::new(
        "use crate::shape::Shape;\n\nfn area(shape: Shape) -> u8 {\n    match shape {\n        Shape::Square => grow(1),\n        _ => 2,\n    }\n}\n",
    ),
);
const ROUND_X: Source =
    Source::new(Located::new("crates/x/src/shape.rs"), Text::new("pub enum Shape {\n    Round { radius: u8 },\n}\n"));
const STRUCT_ARM_X: Source = Source::new(
    Located::new("crates/x/src/area.rs"),
    Text::new(
        "use crate::shape::Shape;\n\nfn area(shape: Shape) -> u8 {\n    match shape {\n        Shape::Round { radius } => grow(radius),\n    }\n}\n",
    ),
);
const PORTS: Source =
    Source::new(Located::new("crates/ports/src/lib.rs"), Text::new("mod watch;\n\npub use watch::{Stall, Watched};\n"));
const WATCH: Source = Source::new(
    Located::new("crates/ports/src/watch.rs"),
    Text::new("pub enum Watched {\n    Stalled(u8),\n    Finished,\n}\n"),
);
const USER: Source = Source::new(
    Located::new("crates/user/src/lib.rs"),
    Text::new(
        "use ports::{Other, Watched};\n\nimpl Seer {\n    fn seen(&self, watched: Watched) -> Result<u8, F> {\n        match watched {\n            Watched::Stalled(at) => Ok(Step::Ended(at)),\n            Watched::Finished => self.read(1),\n        }\n    }\n}\n",
    ),
);

const BOTH_ARMS: Flagged = Flagged::new(&["crates/x/src/area.rs:5", "crates/x/src/area.rs:6"]);

const CASES: [Case; 20] = [
    Case::new(Title::new("a sum type of another file of the crate"), &[LIB_X, SUM_SHAPE_X, AREA_X], BOTH_ARMS),
    Case::new(
        Title::new("a re-exported sum type of another crate"),
        &[PORTS, WATCH, USER],
        Flagged::new(&["crates/user/src/lib.rs:6", "crates/user/src/lib.rs:7"]),
    ),
    Case::new(Title::new("a match in a test module"), &[LIB_X, SUM_SHAPE_X, TEST_AREA_X], Flagged::new(&[])),
    Case::new(
        Title::new("an enum in an inline module"),
        &[LIB_X, INLINE_SHAPE_X, INLINE_AREA_X],
        Flagged::new(&["crates/x/src/area.rs:5"]),
    ),
    Case::new(Title::new("a unit enum elsewhere"), &[LIB_X, COLOR_X, PAINT_X], Flagged::new(&[])),
    Case::new(Title::new("arms that only tabulate"), &[LIB_X, WIDE_SHAPE_X, TABLES_X], Flagged::new(&[])),
    Case::new(
        Title::new("a match in a method"),
        &[LIB_X, SUM_SHAPE_X, METHOD_X],
        Flagged::new(&["crates/x/src/area.rs:6"]),
    ),
    Case::new(
        Title::new("a match in an inline module"),
        &[LIB_X, SUM_SHAPE_X, MODULE_X],
        Flagged::new(&["crates/x/src/area.rs:6"]),
    ),
    Case::new(Title::new("if let"), &[LIB_X, SUM_SHAPE_X, IF_LET_X], Flagged::new(&["crates/x/src/area.rs:4"])),
    Case::new(Title::new("let else"), &[LIB_X, SUM_SHAPE_X, LET_ELSE_X], Flagged::new(&["crates/x/src/area.rs:4"])),
    Case::new(
        Title::new("a unit variant arm"),
        &[LIB_X, SUM_SHAPE_X, UNIT_ARM_X],
        Flagged::new(&["crates/x/src/area.rs:5"]),
    ),
    Case::new(
        Title::new("a struct variant arm"),
        &[LIB_X, ROUND_X, STRUCT_ARM_X],
        Flagged::new(&["crates/x/src/area.rs:5"]),
    ),
    Case::new(
        Title::new("the crate's own unit enum, the other crate's sum type read first"),
        &[LIB_Y, SUM_SHAPE_Y, LIB_X, UNIT_SHAPE_X, UNIT_AREA_X],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("the crate's own unit enum, the other crate's sum type read last"),
        &[LIB_X, UNIT_SHAPE_X, UNIT_AREA_X, LIB_Y, SUM_SHAPE_Y],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("the crate's own sum type, the other crate's unit enum read first"),
        &[LIB_Y, UNIT_SHAPE_Y, LIB_X, SUM_SHAPE_X, AREA_X],
        BOTH_ARMS,
    ),
    Case::new(
        Title::new("the crate's own sum type, the other crate's unit enum read last"),
        &[LIB_X, SUM_SHAPE_X, AREA_X, LIB_Y, UNIT_SHAPE_Y],
        BOTH_ARMS,
    ),
    Case::new(
        Title::new("two sum types of one name, the other crate's read first"),
        &[LIB_Y, SUM_SHAPE_Y, LIB_X, SUM_SHAPE_X, AREA_X],
        BOTH_ARMS,
    ),
    Case::new(
        Title::new("two sum types of one name, the other crate's read last"),
        &[LIB_X, SUM_SHAPE_X, AREA_X, LIB_Y, SUM_SHAPE_Y],
        BOTH_ARMS,
    ),
    Case::new(
        Title::new("the file's own sum type, the other crate's read first"),
        &[LIB_Y, SUM_SHAPE_Y, LIB_X, OWN_AREA_X],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("the file's own sum type, the other crate's read last"),
        &[LIB_X, OWN_AREA_X, LIB_Y, SUM_SHAPE_Y],
        Flagged::new(&[]),
    ),
];

#[test]
fn foreign_matches_are_flagged() {
    assert_eq!(Sources::mismatches::<Queried<ForeignMatches>>(&CASES), Lines::default());
}
