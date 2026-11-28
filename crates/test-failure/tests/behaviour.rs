use std::fs;

use tempfile::TempDir;
use test_failure::{Failed, Golden};

#[test]
fn a_golden_holds_exactly_when_its_file_ends_up_saying_what_was_shown() -> Result<(), Failed> {
    let directory = TempDir::new()?;
    let path = directory.path().join("golden.txt");
    fs::write(&path, "recorded")?;
    let held = Golden::at(directory.path(), "golden.txt").checked("shown");
    assert_eq!(held.is_ok(), fs::read_to_string(&path)? == "shown");
    Ok(())
}
