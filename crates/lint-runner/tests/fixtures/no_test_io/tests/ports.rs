use std::fs;
#[test]
fn writes() {
    fs::write("a", "b").ok();
}
