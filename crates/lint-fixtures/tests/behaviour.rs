use lint_core::{Fix, Headline, Lint, Message, Problem, Report, Site, SourceFile, Violation};
use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};
use std::path::Path;

const SOURCES: &[Source] = &[Source::new(Located::new("a.rs"), Text::new("struct A;\n"))];

const MANIFESTS: &[Source] = &[Source::new(
    Located::new("crates/a/Cargo.toml"),
    Text::new("[package]\nname = \"a\"\n\n[dev-dependencies]\nlogos = \"1\"\n"),
)];
const PLACED: &[Source] = &[Source::new(Located::new("crates/a/src/lib.rs"), Text::new("struct A;\n"))];

struct EveryFile;

struct EveryDependency;

impl Lint for EveryDependency {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let flagged = files.iter().flat_map(|file| {
            file.head().placement().package().dependency_sites().sites(["logos"]).into_iter().map(|line| {
                Violation::new(
                    Site::new(file.head().manifest_file().as_ref(), line),
                    Message::new(Problem::new("depends"), Fix::new("drop it")),
                )
            })
        });
        Report::new(Headline::new("every dependency"), flagged.collect())
    }
}

impl Lint for EveryFile {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let flagged = files.iter().map(|file| {
            Violation::new(
                Site::of_file(AsRef::<Path>::as_ref(file.head())),
                Message::new(Problem::new("flagged"), Fix::new("unflag it")),
            )
        });
        Report::new(Headline::new("every file"), flagged.collect())
    }
}

#[test]
fn a_case_mismatches_only_when_its_lines_differ() {
    let wanted = [Case::new(Title::new("wanted"), SOURCES, Flagged::new(&["a.rs:1"]))];
    let unwanted = [Case::new(Title::new("unwanted"), SOURCES, Flagged::new(&[]))];
    assert_eq!(Sources::mismatches::<EveryFile>(&wanted), Lines::default());
    assert_eq!(Sources::mismatches::<EveryFile>(&unwanted), Lines::from(["unwanted (a.rs ) flagged [\"a.rs:1\"]"]));
}

#[test]
fn lines_are_not_equal_to_other_lines() {
    assert_ne!(Sources::flagged::<EveryFile>(SOURCES), Lines::from(["b.rs:1"]));
}

#[test]
fn placed_sources_read_their_package_manifest() {
    assert_eq!(Sources::placed_flagged::<EveryDependency>(MANIFESTS, PLACED), Lines::from(["crates/a/Cargo.toml:5"]));
    assert_eq!(Sources::placed_flagged::<EveryDependency>(&[], PLACED), Lines::default());
}
