use std::collections::BTreeMap;
use std::fs;

use tempfile::TempDir;
use test_failure::{Cases, Failed, Golden};

#[test]
fn a_golden_holds_exactly_when_its_file_ends_up_saying_what_was_shown() -> Result<(), Failed> {
    let directory = TempDir::new()?;
    let path = directory.path().join("golden.txt");
    fs::write(&path, "recorded")?;
    let held = Golden::at(directory.path(), "golden.txt").checked("shown");
    assert_eq!(held.is_ok(), fs::read_to_string(&path)? == "shown");
    Ok(())
}

#[test]
fn a_golden_or_case_naming_a_directory_of_the_machine_running_it_is_refused() -> Result<(), Failed> {
    let directory = TempDir::new()?;
    let shown = format!("answer = {:?}\n", directory.path().join("x"));
    let path = directory.path().join("cases.toml");
    fs::write(&path, &shown)?;
    let checked = Golden::at(directory.path(), "cases.toml").checked(&shown);
    let read = Cases::read::<BTreeMap<String, String>>(&path);
    assert_eq!((checked.is_err(), read.is_err()), (true, true));
    Ok(())
}
