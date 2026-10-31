mod common;

use common::Sources;
use lints::VecOutParams;

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
