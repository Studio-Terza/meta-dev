use std::fs;
fn f() {
    fs::remove_dir("x");
    fs::read("x");
}
