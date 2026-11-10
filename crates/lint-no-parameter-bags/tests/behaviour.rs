use lint_no_parameter_bags::NoParameterBags;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

const BAG: Source =
    Source::new(Located::new("crates/a/src/bag.rs"), Text::new("pub struct Options {\n    pub depth: u8,\n}\n"));
const FIRST_USER: Source =
    Source::new(Located::new("crates/a/src/first.rs"), Text::new("fn run(options: Options) {}\n"));

const PRIVATE_BAG: Source =
    Source::new(Located::new("crates/a/src/bag.rs"), Text::new("struct Options {\n    depth: u8,\n}\n"));
const EXPORTED_BAG: Source =
    Source::new(Located::new("crates/a/src/bag.rs"), Text::new("pub struct Options {\n    pub depth: u8,\n}\n"));
const ROOT: Source = Source::new(Located::new("crates/a/src/lib.rs"), Text::new("mod bag;\npub use bag::Options;\n"));
const PRIVATE_ROOT: Source =
    Source::new(Located::new("crates/a/src/lib.rs"), Text::new("mod bag;\nuse bag::Options;\n"));
const INNER_USER: Source =
    Source::new(Located::new("crates/a/src/first.rs"), Text::new("fn run(options: Options) {}\n"));
const OTHER_CRATE_USER: Source =
    Source::new(Located::new("crates/b/src/lib.rs"), Text::new("fn run(options: Options) {}\n"));

const ACCESSORS: Source = Source::new(
    Located::new("crates/a/src/bag.rs"),
    Text::new(
        "pub struct Options {\n    depth: u8,\n}\nimpl Options {\n    pub fn new(depth: u8) -> Options {\n        Options { depth }\n    }\n    pub fn empty() -> Self {\n        Self { depth: 1 }\n    }\n    pub fn depth(&self) -> u8 {\n        self.depth\n    }\n}\n",
    ),
);
const READS_ANOTHER: Source = Source::new(
    Located::new("crates/a/src/bag.rs"),
    Text::new(
        "pub struct Options {\n    depth: u8,\n}\nimpl Options {\n    pub fn of(other: Deep) -> u8 {\n        other.depth\n    }\n}\n",
    ),
);
const BUILDS_ANOTHER: Source = Source::new(
    Located::new("crates/a/src/bag.rs"),
    Text::new(
        "pub struct Options {\n    depth: u8,\n}\nimpl Options {\n    pub fn build() -> Other {\n        Other { depth: 1 }\n    }\n}\n",
    ),
);
const TWO_STATEMENTS: Source = Source::new(
    Located::new("crates/a/src/bag.rs"),
    Text::new(
        "pub struct Options {\n    depth: u8,\n}\nimpl Options {\n    pub fn depth(&self) -> u8 {\n        let depth = self.depth;\n        depth\n    }\n}\n",
    ),
);
const TRAIT_IMPL: Source = Source::new(
    Located::new("crates/a/src/bag.rs"),
    Text::new("pub struct Options {\n    depth: u8,\n}\nimpl Clone for Options {\n}\n"),
);
const INLINE: Source = Source::new(
    Located::new("crates/a/src/bag.rs"),
    Text::new("mod inner {\n    pub struct Options {\n        pub depth: u8,\n    }\n}\n"),
);

const CASES: [Case; 11] = [
    Case::new(
        Title::new("another crate's namesake is no consumer"),
        &[PRIVATE_BAG, INNER_USER, OTHER_CRATE_USER],
        Flagged::new(&["crates/a/src/bag.rs:1"]),
    ),
    Case::new(Title::new("exported from the library root"), &[EXPORTED_BAG, ROOT], Flagged::new(&[])),
    Case::new(
        Title::new("imported but not exported"),
        &[EXPORTED_BAG, PRIVATE_ROOT],
        Flagged::new(&["crates/a/src/bag.rs:1"]),
    ),
    Case::new(
        Title::new("exported and used elsewhere"),
        &[EXPORTED_BAG, ROOT, INNER_USER, OTHER_CRATE_USER],
        Flagged::new(&[]),
    ),
    Case::new(Title::new("no consumer"), &[BAG], Flagged::new(&[])),
    Case::new(
        Title::new("accessors and constructors are no logic"),
        &[ACCESSORS, FIRST_USER],
        Flagged::new(&["crates/a/src/bag.rs:1"]),
    ),
    Case::new(Title::new("reading another value"), &[READS_ANOTHER, FIRST_USER], Flagged::new(&[])),
    Case::new(Title::new("building another type"), &[BUILDS_ANOTHER, FIRST_USER], Flagged::new(&[])),
    Case::new(Title::new("two statements"), &[TWO_STATEMENTS, FIRST_USER], Flagged::new(&[])),
    Case::new(Title::new("a trait impl"), &[TRAIT_IMPL, FIRST_USER], Flagged::new(&[])),
    Case::new(Title::new("inline module"), &[INLINE, FIRST_USER], Flagged::new(&["crates/a/src/bag.rs:2"])),
];

#[test]
fn structs_without_logic_that_one_file_consumes_are_flagged() {
    assert_eq!(Sources::mismatches::<NoParameterBags>(&CASES), Lines::default());
}
