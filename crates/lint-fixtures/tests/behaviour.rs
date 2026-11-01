use lint_core::{Headline, Lint, Report, Site, SourceFile, Violation};
use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

const SOURCES: &[Source] = &[Source::new(Located::new("a.rs"), Text::new("struct A;\n"))];

struct EveryFile;

impl Lint for EveryFile {
    fn report(files: &[SourceFile]) -> Report {
        let flagged = files.iter().map(|file| Violation::new(Site::of_file(file.path()), "flagged"));
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
