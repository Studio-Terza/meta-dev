use std::path::PathBuf;

use code_reader::Reader;
use lint_core::SourceFile;
use lint_fixtures::{Sources, Text};
use lint_placement::CrateName;
use lint_placement::CrateRoot;
use lint_test_budget::TestBudget;

const SHIPPED: &str = "pub struct A;\npub struct B;\n";
const WITH_MODULE: &str = "pub struct A;\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {}\n}\n";
const NESTED: &str = "#[cfg(test)]\nmod a {\n    mod b {\n        #[test]\n        fn t() {}\n    }\n}\n";
const TESTS: &str = "#[test]\nfn a() {}\n#[tokio::test]\nfn b() {}\nfn helper() {}\n";
const STARTS: &str = "use std::process::Command;\nfn f() {\n    Command::new(\"ls\").status();\n}\nmod ports {\n    fn g() {\n        std::process::Command::new(\"ls\").spawn();\n    }\n}\n";
const LAWS: &str = "laws::table!(a, f, [1 => 2]);\nroundtrip!(b, s, e, d);\nlaws::idempotent!(c, s, f);\n";
const OTHER_MACRO: &str = "wrap! {\n    #[test]\n    fn a() {}\n}\n";
const PROPTEST: &str = "proptest! {\n    #[test]\n    fn a(x in 0..2) {}\n    #[test]\n    fn b(x in 0..2) {}\n}\n";

struct Budgets;

impl Budgets {
    fn of_crate(files: &[SourceFile], name: &str) -> Option<String> {
        let crate_name = CrateName::from(&CrateRoot::new(PathBuf::from(name)));
        TestBudget::measured(files, &Reader::read_files(files)).of_crate(&crate_name).map(|budget| budget.to_string())
    }

    fn of_test_file(text: &&str) -> Option<String> {
        Self::of_crate(&[Sources::file("crates/a/tests/t.rs", Text::new(text))], "a")
    }

    fn counted(tests: usize, files: usize, lines: usize, test_lines: usize) -> String {
        Self::started(tests, files, lines, test_lines, 0)
    }

    fn started(tests: usize, files: usize, lines: usize, test_lines: usize, starts: usize) -> String {
        format!("{tests} tests in {files} files, {test_lines} of {lines} lines, {starts} process starts")
    }
}

laws::table!(
    tests_files_and_lines_are_counted,
    Budgets::of_test_file,
    [
        TESTS => Some(Budgets::counted(2, 1, 5, 5)),
        LAWS => Some(Budgets::counted(3, 1, 3, 3)),
        PROPTEST => Some(Budgets::counted(2, 1, 6, 6)),
        OTHER_MACRO => Some(Budgets::counted(0, 1, 4, 4)),
    ]
);

#[test]
fn only_test_modules_count_as_test_lines_and_only_once() {
    let shipped = [Sources::file("crates/a/src/lib.rs", Text::new(SHIPPED))];
    assert_eq!(Budgets::of_crate(&shipped, "a"), Some(Budgets::counted(0, 1, 2, 0)));
    let with_module = [Sources::file("crates/a/src/lib.rs", Text::new(WITH_MODULE))];
    assert_eq!(Budgets::of_crate(&with_module, "a"), Some(Budgets::counted(1, 1, 6, 5)));
    let nested = [Sources::file("crates/a/src/lib.rs", Text::new(NESTED))];
    assert_eq!(Budgets::of_crate(&nested, "a"), Some(Budgets::counted(1, 1, 7, 7)));
}

#[test]
fn build_scripts_and_process_starts_belong_to_their_own_package() {
    let with_build = [
        Sources::file("crates/a/src/lib.rs", Text::new(SHIPPED)),
        Sources::file("crates/a/build.rs", Text::new(SHIPPED)),
    ];
    assert_eq!(Budgets::of_crate(&with_build, "a"), Some(Budgets::counted(0, 2, 4, 0)));
    let starting = [
        Sources::file("crates/a/src/lib.rs", Text::new(STARTS)),
        Sources::file("crates/b/src/lib.rs", Text::new(SHIPPED)),
    ];
    assert_eq!(Budgets::of_crate(&starting, "b"), Some(Budgets::counted(0, 1, 2, 0)));
}

#[test]
fn a_test_module_file_counts_entirely_as_test_lines() {
    let module = Sources::file("crates/a/src/tests.rs", Text::new(TESTS));
    let files = [Sources::file("crates/a/src/lib.rs", Text::new(SHIPPED)).with_test_modules(vec![module])];
    assert_eq!(Budgets::of_crate(&files, "a"), Some(Budgets::counted(2, 2, 7, 5)));
}

#[test]
fn files_of_a_package_add_up_and_packages_stay_apart() {
    let files = [
        Sources::file("crates/a/src/lib.rs", Text::new(SHIPPED)),
        Sources::file("crates/a/tests/t.rs", Text::new(TESTS)),
        Sources::file("crates/b/src/lib.rs", Text::new(SHIPPED)),
    ];
    assert_eq!(Budgets::of_crate(&files, "a"), Some(Budgets::counted(2, 2, 7, 5)));
    assert_eq!(Budgets::of_crate(&files, "b"), Some(Budgets::counted(0, 1, 2, 0)));
}

#[test]
fn the_report_lists_each_crate() {
    let files = [Sources::file("crates/a/tests/t.rs", Text::new(TESTS))];
    let report = TestBudget::measured(&files, &Reader::read_files(&files)).to_string();
    assert!(report.contains("a: "));
    assert!(report.contains("report only"));
}

#[test]
fn process_starts_are_counted_inside_and_outside_ports() {
    let files = [Sources::file("crates/a/src/lib.rs", Text::new(STARTS))];
    assert_eq!(Budgets::of_crate(&files, "a"), Some(Budgets::started(0, 1, 9, 0, 2)));
}
