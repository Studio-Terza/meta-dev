use std::path::PathBuf;

use lint_core::SourceFile;
use lint_fixtures::Sources;
use lint_placement::{CrateName, CrateRoot};
use lint_test_budget::{Count, CrateBudget, TestBudget};

const SHIPPED: &str = "pub struct A;\npub struct B;\n";
const WITH_MODULE: &str = "pub struct A;\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {}\n}\n";
const NESTED: &str = "#[cfg(test)]\nmod a {\n    mod b {\n        #[test]\n        fn t() {}\n    }\n}\n";
const TESTS: &str = "#[test]\nfn a() {}\n#[tokio::test]\nfn b() {}\nfn helper() {}\n";
const LAWS: &str = "laws::table!(a, f, [1 => 2]);\nroundtrip!(b, s, e, d);\nlaws::idempotent!(c, s, f);\n";
const PROPTEST: &str = "proptest! {\n    #[test]\n    fn a(x in 0..2) {}\n    #[test]\n    fn b(x in 0..2) {}\n}\n";

struct Budgets;

impl Budgets {
    fn of_crate(files: &[SourceFile], name: &str) -> Option<CrateBudget> {
        let name = CrateName::from(&CrateRoot::new(PathBuf::from(name)));
        TestBudget::measured(files).of_crate(&name).copied()
    }

    fn of_test_file(text: &&str) -> Option<CrateBudget> {
        Self::of_crate(&[Sources::file("crates/a/tests/t.rs", text)], "a")
    }

    fn counted(tests: usize, files: usize, lines: usize, test_lines: usize) -> CrateBudget {
        CrateBudget::new(Count::from(tests), Count::from(files), Count::from(lines), Count::from(test_lines))
    }
}

laws::table!(
    tests_files_and_lines_are_counted,
    Budgets::of_test_file,
    [
        TESTS => Some(Budgets::counted(2, 1, 5, 5)),
        LAWS => Some(Budgets::counted(3, 1, 3, 3)),
        PROPTEST => Some(Budgets::counted(2, 1, 6, 6)),
    ]
);

#[test]
fn shipped_lines_are_not_test_lines() {
    let files = [Sources::file("crates/a/src/lib.rs", SHIPPED)];
    assert_eq!(Budgets::of_crate(&files, "a"), Some(Budgets::counted(0, 1, 2, 0)));
}

#[test]
fn an_inline_test_module_counts_as_test_lines() {
    let files = [Sources::file("crates/a/src/lib.rs", WITH_MODULE)];
    assert_eq!(Budgets::of_crate(&files, "a"), Some(Budgets::counted(1, 1, 6, 5)));
}

#[test]
fn nested_test_modules_count_their_lines_once() {
    let files = [Sources::file("crates/a/src/lib.rs", NESTED)];
    assert_eq!(Budgets::of_crate(&files, "a"), Some(Budgets::counted(1, 1, 7, 7)));
}

#[test]
fn a_build_script_belongs_to_its_package() {
    let files = [Sources::file("crates/a/src/lib.rs", SHIPPED), Sources::file("crates/a/build.rs", SHIPPED)];
    assert_eq!(Budgets::of_crate(&files, "a"), Some(Budgets::counted(0, 2, 4, 0)));
}

#[test]
fn a_test_module_file_counts_entirely_as_test_lines() {
    let module = Sources::file("crates/a/src/tests.rs", TESTS);
    let files = [Sources::file("crates/a/src/lib.rs", SHIPPED).with_test_modules(vec![module])];
    assert_eq!(Budgets::of_crate(&files, "a"), Some(Budgets::counted(2, 2, 7, 5)));
}

#[test]
fn files_of_a_package_add_up_and_packages_stay_apart() {
    let files = [
        Sources::file("crates/a/src/lib.rs", SHIPPED),
        Sources::file("crates/a/tests/t.rs", TESTS),
        Sources::file("crates/b/src/lib.rs", SHIPPED),
    ];
    assert_eq!(Budgets::of_crate(&files, "a"), Some(Budgets::counted(2, 2, 7, 5)));
    assert_eq!(Budgets::of_crate(&files, "b"), Some(Budgets::counted(0, 1, 2, 0)));
}

#[test]
fn the_report_lists_each_crate() {
    let report = TestBudget::measured(&[Sources::file("crates/a/tests/t.rs", TESTS)]).to_string();
    assert!(report.contains("a: ") && report.contains("report only"));
}
