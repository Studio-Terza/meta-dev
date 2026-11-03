use std::process::Command;
#[test]
fn starts() {
    Command::new("ls").status().ok();
}
