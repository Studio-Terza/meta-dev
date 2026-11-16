use lint_no_parameter_bags::NoParameterBags;

use lint_fixtures::{Case, Flagged, Lines, Located, Queried, Source, Sources, Text, Title};

const ROOT: Source = Source::new(Located::new("crates/a/src/lib.rs"), Text::new("mod bag;\nmod first;\n"));
const EXPORTING_ROOT: Source =
    Source::new(Located::new("crates/a/src/lib.rs"), Text::new("mod bag;\nmod first;\npub use bag::Options;\n"));
const IMPORTING_ROOT: Source =
    Source::new(Located::new("crates/a/src/lib.rs"), Text::new("mod bag;\nmod first;\nuse bag::Options;\n"));
const NAMESAKE_ROOT: Source =
    Source::new(Located::new("crates/a/src/lib.rs"), Text::new("mod bag;\nmod first;\nmod other;\nmod second;\n"));
const LONE_ROOT: Source = Source::new(Located::new("crates/a/src/lib.rs"), Text::new("mod bag;\n"));

const BAG: Source =
    Source::new(Located::new("crates/a/src/bag.rs"), Text::new("pub struct Options {\n    pub depth: u8,\n}\n"));
const FIRST_USER: Source = Source::new(
    Located::new("crates/a/src/first.rs"),
    Text::new("use crate::bag::Options;\npub struct Runner;\nimpl Runner {\n    pub fn run(options: Options) {}\n}\n"),
);
const BODY_USER: Source = Source::new(
    Located::new("crates/a/src/first.rs"),
    Text::new("pub struct Runner;\nimpl Runner {\n    pub fn run() -> u8 {\n        let options: Option<crate::bag::Options> = None;\n        let built = crate::bag::Options { depth: 1 };\n        built.depth\n    }\n}\n"),
);
const OTHER_CRATE_USER: Source = Source::new(
    Located::new("crates/b/src/lib.rs"),
    Text::new("pub struct Runner;\nimpl Runner {\n    pub fn run(options: Options) {}\n}\n"),
);
const NAMESAKE: Source = Source::new(
    Located::new("crates/a/src/other.rs"),
    Text::new("pub enum Kind {\n    Options,\n}\npub struct Options;\n"),
);
const NAMESAKE_USER: Source = Source::new(
    Located::new("crates/a/src/second.rs"),
    Text::new(
        "use crate::other::{Kind, Options};\npub struct Walker;\nimpl Walker {\n    pub fn walk(options: Options) -> Kind {\n        Kind::Options\n    }\n}\n",
    ),
);

const ACCESSORS: Source = Source::new(
    Located::new("crates/a/src/bag.rs"),
    Text::new(
        "pub struct Options {\n    depth: u8,\n}\nimpl Options {\n    pub fn new(depth: u8) -> Options {\n        Options { depth }\n    }\n    pub fn empty() -> Self {\n        Self { depth: 1 }\n    }\n    pub fn depth(&self) -> u8 {\n        self.depth\n    }\n}\n",
    ),
);
const READS_ANOTHER: Source = Source::new(
    Located::new("crates/a/src/bag.rs"),
    Text::new(
        "pub struct Options {\n    depth: u8,\n}\npub struct Deep {\n    depth: u8,\n}\nimpl Options {\n    pub fn deep(other: Deep) -> u8 {\n        other.depth\n    }\n}\n",
    ),
);
const BUILDS_ANOTHER: Source = Source::new(
    Located::new("crates/a/src/bag.rs"),
    Text::new(
        "pub struct Options {\n    depth: u8,\n}\npub struct Other {\n    depth: u8,\n}\nimpl Options {\n    pub fn build() -> Other {\n        Other { depth: 1 }\n    }\n}\n",
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
const DERIVED: Source = Source::new(
    Located::new("crates/a/src/bag.rs"),
    Text::new("#[derive(Clone)]\npub struct Options {\n    depth: u8,\n}\n"),
);
const INLINE: Source = Source::new(
    Located::new("crates/a/src/bag.rs"),
    Text::new("pub mod inner {\n    pub struct Options {\n        pub depth: u8,\n    }\n}\n"),
);
const INLINE_USER: Source = Source::new(
    Located::new("crates/a/src/first.rs"),
    Text::new(
        "use crate::bag::inner::Options;\npub struct Runner;\nimpl Runner {\n    pub fn run(options: Options) {}\n}\n",
    ),
);

const CASES: [Case; 14] = [
    Case::new(
        Title::new("one other file consumes it"),
        &[ROOT, BAG, FIRST_USER],
        Flagged::new(&["crates/a/src/bag.rs:1"]),
    ),
    Case::new(
        Title::new("a body that annotates and builds it consumes it"),
        &[ROOT, BAG, BODY_USER],
        Flagged::new(&["crates/a/src/bag.rs:1"]),
    ),
    Case::new(
        Title::new("another crate's namesake is no consumer"),
        &[ROOT, BAG, FIRST_USER, OTHER_CRATE_USER],
        Flagged::new(&["crates/a/src/bag.rs:1"]),
    ),
    Case::new(
        Title::new("a namesake type and variant in another module are no consumers"),
        &[NAMESAKE_ROOT, BAG, FIRST_USER, NAMESAKE, NAMESAKE_USER],
        Flagged::new(&["crates/a/src/bag.rs:1", "crates/a/src/other.rs:4"]),
    ),
    Case::new(Title::new("exported from the library root"), &[EXPORTING_ROOT, BAG, FIRST_USER], Flagged::new(&[])),
    Case::new(
        Title::new("imported at the root but not exported"),
        &[IMPORTING_ROOT, BAG, FIRST_USER],
        Flagged::new(&["crates/a/src/bag.rs:1"]),
    ),
    Case::new(Title::new("no consumer"), &[LONE_ROOT, BAG], Flagged::new(&[])),
    Case::new(
        Title::new("accessors and constructors are no logic"),
        &[ROOT, ACCESSORS, FIRST_USER],
        Flagged::new(&["crates/a/src/bag.rs:1"]),
    ),
    Case::new(Title::new("reading another value"), &[ROOT, READS_ANOTHER, FIRST_USER], Flagged::new(&[])),
    Case::new(Title::new("building another type"), &[ROOT, BUILDS_ANOTHER, FIRST_USER], Flagged::new(&[])),
    Case::new(Title::new("two statements"), &[ROOT, TWO_STATEMENTS, FIRST_USER], Flagged::new(&[])),
    Case::new(Title::new("a trait impl"), &[ROOT, TRAIT_IMPL, FIRST_USER], Flagged::new(&[])),
    Case::new(
        Title::new("a derive is no logic"),
        &[ROOT, DERIVED, FIRST_USER],
        Flagged::new(&["crates/a/src/bag.rs:2"]),
    ),
    Case::new(Title::new("inline module"), &[ROOT, INLINE, INLINE_USER], Flagged::new(&["crates/a/src/bag.rs:2"])),
];

#[test]
fn structs_without_logic_that_one_file_consumes_are_flagged() {
    assert_eq!(Sources::mismatches::<Queried<NoParameterBags>>(&CASES), Lines::default());
}
