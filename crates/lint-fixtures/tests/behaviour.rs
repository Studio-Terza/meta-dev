use lint_core::{Fix, Headline, Lint, Message, Problem, Report, Site, SourceFile, Violation};
use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

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
            file.placement().package().dependencies().as_ref().iter().map(|dependency| {
                Violation::new(
                    Site::new(file.manifest_file().as_ref(), dependency.line()),
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
            Violation::new(Site::of_file(file.path()), Message::new(Problem::new("flagged"), Fix::new("unflag it")))
        });
        Report::new(Headline::new("every file"), flagged.collect())
    }
}

laws::table!(a_case_mismatches_only_when_its_lines_differ, Sources::mismatches::<EveryFile>, [
    [Case::new(Title::new("wanted"), SOURCES, Flagged::new(&["a.rs:1"]))] => Lines::default(),
    [Case::new(Title::new("unwanted"), SOURCES, Flagged::new(&[]))] => ["unwanted (a.rs ) flagged [\"a.rs:1\"]"],
]);

#[test]
fn lines_are_not_equal_to_other_lines() {
    assert_ne!(Sources::flagged::<EveryFile>(SOURCES), ["b.rs:1"]);
}

impl EveryDependency {
    fn flagged([manifests, sources]: &[&[Source]; 2]) -> Lines {
        Sources::placed_flagged::<EveryDependency>(manifests, sources)
    }
}

laws::table!(placed_sources_read_their_package_manifest, EveryDependency::flagged, [
    [MANIFESTS, PLACED] => ["crates/a/Cargo.toml:5"],
    [&[], PLACED] => Lines::default(),
]);
