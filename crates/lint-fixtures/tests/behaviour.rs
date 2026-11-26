use lint_core::{Fix, Headline, Lint, Message, Problem, Report, Site, SourceFile, Violation};
use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};
use std::path::Path;

const SOURCES: &[Source] = &[Source::new(Located::new("a.rs"), Text::new("struct A;\n"))];

struct EveryFile;

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
