use lint_core::Lint;
use lint_fixtures::Sources;
use lint_vec_out_params::VecOutParams;

const OUT: &str =
    "fn fill(out: &mut Vec<u8>) {}\nstruct A;\nimpl A {\n    fn push(&mut self, out: &mut Vec<u8>) {}\n}\n";
const FINE: &str = "fn read(items: &Vec<u8>) {}\nfn take(items: Vec<u8>) {}\n#[cfg(test)]\nmod tests {\n    fn fill(out: &mut Vec<u8>) {}\n}\n";

#[test]
fn mutable_vec_parameters_are_reported() {
    let report = VecOutParams::report(&[Sources::file("a.rs", OUT)]);
    assert_eq!(Sources::lines(&report), ["a.rs:1", "a.rs:4"]);
}

#[test]
fn shared_and_owned_vecs_and_test_modules_pass() {
    assert!(VecOutParams::report(&[Sources::file("a.rs", FINE)]).is_clean());
}

const TRAIT_IMPL: &str = "impl Read for R {\n    fn read_to_end(&mut self, buf: &mut Vec<u8>) {}\n}\n";
const OPTIONAL: &str = "pub fn g(out: Option<&mut Vec<u8>>) {}\n";
const IN_MACRO: &str = "macro_rules! m { () => { fn free(x: &mut Vec<u8>) { } } }\n";

#[test]
fn foreign_trait_impls_are_not_reported() {
    assert!(VecOutParams::report(&[Sources::file("a.rs", TRAIT_IMPL)]).is_clean());
}

#[test]
fn optional_mutable_vecs_are_reported() {
    let report = VecOutParams::report(&[Sources::file("a.rs", OPTIONAL)]);
    assert_eq!(Sources::lines(&report), ["a.rs:1"]);
}

#[test]
fn macro_rules_bodies_are_checked() {
    let report = VecOutParams::report(&[Sources::file("a.rs", IN_MACRO)]);
    assert_eq!(Sources::lines(&report), ["a.rs:1"]);
}
