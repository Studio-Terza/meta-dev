use lint_derived_tables::DerivedTables;
use lint_file::Ruled;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

const DOOR: &str = "enum Door {\n    Open,\n    Shut,\n}\n";
const EVERY: &str = "enum Door {\n    Open,\n    Shut,\n}\n\nconst ALL: [Door; 2] = [Door::Open, Door::Shut];\n";
const SOME: &str =
    "enum Door {\n    Open,\n    Shut,\n    Ajar,\n}\n\nconst CLOSABLE: [Door; 2] = [Door::Open, Door::Ajar];\n";
const ELSEWHERE: &str = "const ALL: &[Door] = &[Door::Shut, Door::Open];\n";
const SELF_LISTED: &str =
    "enum Door {\n    Open,\n    Shut,\n}\n\nimpl Door {\n    const ALL: [Door; 2] = [Self::Open, Self::Shut];\n}\n";
const SPELLED: &str = "impl Door {\n    const NAMED: [(&'static str, Door); 1] = [(\"open\", Door::Open)];\n}\n";
const MAPPED: &str = "impl Door {\n    fn name(self) -> &'static str {\n        match self {\n            Door::Open => \"open\",\n            Door::Shut => \"shut\",\n        }\n    }\n}\n";
const DISPLAYED: &str = "impl fmt::Display for Door {\n    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {\n        match self {\n            Door::Open => formatter.write_str(\"open\"),\n            Door::Shut => write!(formatter, \"shut\"),\n        }\n    }\n}\n";
const COMPUTED: &str = "impl Door {\n    fn name(self) -> String {\n        match self {\n            Door::Open => \"open\",\n            Door::Shut => shut(),\n        }\n    }\n}\n";
const WILDCARD: &str = "impl Door {\n    fn name(self) -> &'static str {\n        match self {\n            Door::Open => \"open\",\n            _ => \"other\",\n        }\n    }\n}\n";
const TESTED: &str = "#[test]\nfn f() {\n    const ALL: [Door; 2] = [Door::Open, Door::Shut];\n}\n";
const GROUPED: &str = "fn name(door: Door) -> &'static str {\n    match door {\n        Door::Open | Door::Ajar => \"open\",\n        Door::Shut => \"shut\",\n    }\n}\n";
const MIXED: &str = "fn name(door: Door) -> &'static str {\n    match door {\n        Door::Open | Gate::Open => \"open\",\n        Door::Shut => \"shut\",\n    }\n}\n";
const SINGLE: &str =
    "fn name(door: Door) -> &'static str {\n    match door {\n        Door::Open => \"open\",\n    }\n}\n";
const BRACED: &str = "fn name(door: Door) -> &'static str {\n    match door {\n        Door::Open => {\n            \"open\"\n        }\n        Door::Shut => {\n            \"shut\"\n        }\n    }\n}\n";

const CASES: [Case; 13] = [
    Case::new(
        Title::new("a fn mapping grouped variants to literals"),
        &[Source::new(Located::new("a.rs"), Text::new(GROUPED))],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(
        Title::new("a fn whose grouped arm spans two enums"),
        &[Source::new(Located::new("a.rs"), Text::new(MIXED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a fn matching a single variant"),
        &[Source::new(Located::new("a.rs"), Text::new(SINGLE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a fn mapping each variant to a literal in a block"),
        &[Source::new(Located::new("a.rs"), Text::new(BRACED))],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(
        Title::new("a const listing every variant"),
        &[Source::new(Located::new("a.rs"), Text::new(EVERY))],
        Flagged::new(&["a.rs:6"]),
    ),
    Case::new(
        Title::new("a const listing some variants"),
        &[Source::new(Located::new("a.rs"), Text::new(SOME))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a const listing every variant of an enum declared in another file"),
        &[Source::new(Located::new("a.rs"), Text::new(DOOR)), Source::new(Located::new("b.rs"), Text::new(ELSEWHERE))],
        Flagged::new(&["b.rs:1"]),
    ),
    Case::new(
        Title::new("a const listing every variant through Self"),
        &[Source::new(Located::new("a.rs"), Text::new(SELF_LISTED))],
        Flagged::new(&["a.rs:7"]),
    ),
    Case::new(
        Title::new("a const pairing spellings with variants"),
        &[Source::new(Located::new("a.rs"), Text::new(SPELLED))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("a fn mapping each variant to a literal"),
        &[
            Source::new(Located::new("a.rs"), Text::new(MAPPED)),
            Source::new(Located::new("b.rs"), Text::new(DISPLAYED)),
        ],
        Flagged::new(&["a.rs:2", "b.rs:2"]),
    ),
    Case::new(
        Title::new("a fn computing one arm"),
        &[Source::new(Located::new("a.rs"), Text::new(COMPUTED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a fn with a wildcard arm"),
        &[Source::new(Located::new("a.rs"), Text::new(WILDCARD))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a test listing every variant"),
        &[
            Source::new(Located::new("a.rs"), Text::new(DOOR)),
            Source::new(Located::new("tests/a.rs"), Text::new(TESTED)),
        ],
        Flagged::new(&["tests/a.rs:3"]),
    ),
];

#[test]
fn hand_kept_variant_lists_spellings_and_literal_maps_are_flagged_and_subsets_are_not() {
    assert_eq!(Sources::mismatches::<Ruled<DerivedTables>>(&CASES), Lines::default());
}
