use lint_core::Lint;
use lint_fixtures::Sources;
use lint_no_free_fns::NoFreeFns;

const FREE: &str = "fn helper() {}\nmod inner {\n    pub fn nested() {}\n}\n";
const ALLOWED: &str = "fn main() {}\n#[test]\nfn check() {}\nstruct A;\nimpl A {\n    fn method() {}\n}\n";

#[test]
fn free_functions_are_reported_including_nested_modules() {
    let report = NoFreeFns::report(&[Sources::file("a.rs", FREE)]);
    assert_eq!(Sources::lines(&report), ["a.rs:1", "a.rs:3"]);
    assert!(report.violations().iter().any(|violation| violation.message() == "fn nested"));
}

#[test]
fn main_tests_and_methods_are_allowed() {
    assert!(NoFreeFns::report(&[Sources::file("a.rs", ALLOWED)]).is_clean());
}

const NESTED: &str = "pub struct A;\nimpl A {\n    pub fn run() {\n        fn helper() {}\n        helper()\n    }\n}\nconst _: () = {\n    fn hidden() {}\n};\n";
const REQUIRED: &str = "#[proc_macro_derive(Foo)]\npub fn foo(input: TokenStream) -> TokenStream { input }\n#[cfg(test)]\nmod tests {\n    #[tokio::test]\n    async fn t() {}\n}\n";

#[test]
fn functions_nested_in_blocks_are_reported() {
    let report = NoFreeFns::report(&[Sources::file("a.rs", NESTED)]);
    assert_eq!(Sources::lines(&report), ["a.rs:4", "a.rs:9"]);
}

#[test]
fn functions_the_language_requires_to_be_free_are_allowed() {
    assert!(NoFreeFns::report(&[Sources::file("a.rs", REQUIRED)]).is_clean());
}
