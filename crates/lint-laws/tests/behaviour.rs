use std::path::PathBuf;

use file_ports::{FileFailure, Files, Memory, Written};
use golden::golden;
use graph_reader::Reader;
use lint_file::ModelLint;
use lint_fixtures::{Lines, Located, Queried, Source, Sources, Text};
use lint_laws::{HandWrittenLaws, LawRunners};

const NEWTYPE: &str = "pub use newtype_derive::Arbitrary;\n";
const ALGEBRA: &str = "pub trait Semigroup {}\npub trait Monoid: Semigroup {}\n";
const LAWS: &str = "#[macro_export]\nmacro_rules! generates {\n    () => {};\n}\n#[macro_export]\nmacro_rules! monoid {\n    () => {};\n}\n";
const ROOT: &str = "mod a;\npub mod b;\npub use a::Total;\nuse newtype::Arbitrary;\nuse algebra::Monoid;\n#[derive(Arbitrary, Debug)]\npub struct SpanWidth(u8);\nimpl Monoid for Total {}\n#[derive(Arbitrary)]\nstruct Hidden(u8);\n";
const MODULE: &str = "#[derive(newtype::Arbitrary)]\npub struct Total(u8);\n";
const PUBLIC_MODULE: &str = "#[derive(newtype::Arbitrary)]\npub struct Nested(u8);\n";
const HAND_WRITTEN: &str = "laws::generates!(total_generates, any::<x::Total>());\n";
const JOINED: &str = "#[derive(newtype::Arbitrary)]\npub struct Joined(u8);\nimpl algebra::Semigroup for Joined {}\n";
const BARE: &str =
    "pub struct Bare;\nimpl algebra::Semigroup for Bare {}\npub struct Empty;\nimpl algebra::Monoid for Empty {}\n";
const HIDDEN: &str = "struct Hidden;\nimpl algebra::Semigroup for Hidden {}\n";

struct Workspace;

impl Workspace {
    fn with(extra: Source) -> Vec<Source> {
        vec![
            Source::new(Located::new("crates/newtype/src/lib.rs"), Text::new(NEWTYPE)),
            Source::new(Located::new("crates/algebra/src/lib.rs"), Text::new(ALGEBRA)),
            Source::new(Located::new("crates/laws/src/lib.rs"), Text::new(LAWS)),
            Source::new(Located::new("crates/x/src/lib.rs"), Text::new(ROOT)),
            Source::new(Located::new("crates/x/src/a.rs"), Text::new(MODULE)),
            Source::new(Located::new("crates/x/src/b.rs"), Text::new(PUBLIC_MODULE)),
            extra,
        ]
    }
}

#[test]
fn each_crate_gets_one_runner_with_a_law_per_nameable_generator() -> Result<(), FileFailure> {
    let model = Reader::read_files(&Sources::parsed(&Workspace::with(Source::new(
        Located::new("crates/x/tests/behaviour.rs"),
        Text::new("fn f() {}\n"),
    ))));
    let written: Vec<Written> = LawRunners::from(&model).written();
    let path = PathBuf::from("crates/x/tests/generated/laws.rs");
    assert_eq!(written.iter().map(Written::path).collect::<Vec<_>>(), [path.as_path()]);
    let disk = Memory::default();
    disk.write_each(written)?;
    let runner = disk.read_text(&path)?;
    golden!("runner.rs.txt", runner.as_ref());
    Ok(())
}

laws::table!(a_generated_law_written_outside_the_runner_is_refused, |sources: &Vec<Source>| Sources::flagged::<Queried<HandWrittenLaws>>(sources), [
    Workspace::with(Source::new(Located::new("crates/x/tests/behaviour.rs"), Text::new(HAND_WRITTEN)))
        => Lines::from(["crates/x/tests/behaviour.rs:1", "crates/x/src/lib.rs:1"]),
    Workspace::with(Source::new(Located::new("crates/x/tests/generated/laws.rs"), Text::new(HAND_WRITTEN)))
        => Lines::default(),
    Workspace::with(Source::new(Located::new("crates/laws/tests/behaviour.rs"), Text::new(HAND_WRITTEN)))
        => Lines::from(["crates/x/src/lib.rs:1"]),
]);

#[test]
fn a_semigroup_that_is_no_monoid_gets_only_associativity() -> Result<(), FileFailure> {
    let model = Reader::read_files(&Sources::parsed(&Workspace::with(Source::new(
        Located::new("crates/y/src/lib.rs"),
        Text::new(JOINED),
    ))));
    let path = PathBuf::from("crates/y/tests/generated/laws.rs");
    let disk = Memory::default();
    disk.write_each(LawRunners::from(&model).written())?;
    let runner = disk.read_text(&path)?;
    golden!("semigroup.rs.txt", runner.as_ref());
    Ok(())
}

laws::table!(a_public_semigroup_the_runner_cannot_generate_is_refused, |sources: &Vec<Source>| Sources::flagged::<Queried<HandWrittenLaws>>(sources), [
    Workspace::with(Source::new(Located::new("crates/y/src/lib.rs"), Text::new(BARE)))
        => Lines::from(["crates/x/src/lib.rs:1", "crates/y/src/lib.rs:1", "crates/y/src/lib.rs:3"]),
    Workspace::with(Source::new(Located::new("crates/y/src/lib.rs"), Text::new(JOINED)))
        => Lines::from(["crates/x/src/lib.rs:1", "crates/y/src/lib.rs:1"]),
    Workspace::with(Source::new(Located::new("crates/y/src/lib.rs"), Text::new(HIDDEN)))
        => Lines::from(["crates/x/src/lib.rs:1"]),
]);

#[test]
fn the_lint_names_what_it_refuses() {
    let model = Reader::read_files(&Sources::parsed(&Workspace::with(Source::new(
        Located::new("crates/x/tests/behaviour.rs"),
        Text::new(HAND_WRITTEN),
    ))));
    golden!("refused.txt", &HandWrittenLaws::report(&model).to_string());
}
