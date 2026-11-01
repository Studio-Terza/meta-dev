use lint_core::{Lint, Report, Site, SourceFile, Violation};
use lint_fixtures::{Case, Source, Sources};

const SOURCES: &[Source] = &[Source::new("a.rs", "struct A;\n")];

struct EveryFile;

impl Lint for EveryFile {
    fn report(files: &[SourceFile]) -> Report {
        let flagged = files.iter().map(|file| Violation::new(Site::of_file(file.path()), "flagged"));
        Report::new("every file", flagged.collect())
    }
}

laws::table!(a_case_mismatches_only_when_its_lines_differ, Sources::mismatches::<EveryFile>, [
    [Case::new("wanted", SOURCES, &["a.rs:1"])] => Vec::<String>::new(),
    [Case::new("unwanted", SOURCES, &[])] => ["unwanted (a.rs ) flagged [\"a.rs:1\"]"],
]);
