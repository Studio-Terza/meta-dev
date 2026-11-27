use std::process::Command;
fn f() {
    Command::new("ls").status();
}
