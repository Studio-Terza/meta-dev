use std::fs;
#[test]
fn reads() {
    fs::read_to_string("a").ok();
}
