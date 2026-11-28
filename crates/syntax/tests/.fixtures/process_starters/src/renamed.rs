use std::process::Command as Run;
fn f() {
    Run::new("ls").output();
}
