use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

use golden::{Difference, Failure, Golden, Mode, golden};
use tempfile::TempDir;

struct Fixture {
    directory: TempDir,
}

impl Fixture {
    const NAME: &str = "nested/output.txt";

    fn new() -> Result<Self, std::io::Error> {
        Ok(Self { directory: TempDir::new()? })
    }

    fn path(&self) -> PathBuf {
        self.directory.path().join(Self::NAME)
    }

    fn golden(&self) -> Golden {
        Golden::located_at(self.path())
    }

    fn with_recorded(text: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let fixture = Self::new()?;
        fixture.golden().verify_in(Mode::Record, text)?;
        Ok(fixture)
    }

    fn text(&self) -> Result<String, std::io::Error> {
        fs::read_to_string(self.path())
    }
}

#[test]
fn checking_matching_output_passes() -> Result<(), Box<dyn std::error::Error>> {
    Fixture::with_recorded("same\n")?.golden().verify_in(Mode::Check, "same\n")?;
    Ok(())
}

#[test]
fn a_check_fails_on_changed_output_and_leaves_the_golden_alone() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::with_recorded("old\n")?;
    assert!(matches!(fixture.golden().verify_in(Mode::Check, "new\n"), Err(Failure::Differs { .. })));
    assert_eq!(fixture.text()?, "old\n");
    Ok(())
}

#[test]
fn a_check_never_creates_a_missing_golden() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    assert!(matches!(fixture.golden().verify_in(Mode::Check, "new\n"), Err(Failure::Missing { .. })));
    assert!(!fixture.path().exists());
    Ok(())
}

#[test]
fn recording_mode_replaces_the_golden() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::with_recorded("old\n")?;
    fixture.golden().verify_in(Mode::Record, "new\n")?;
    assert_eq!(fixture.text()?, "new\n");
    fixture.golden().verify_in(Mode::Check, "new\n")?;
    Ok(())
}

#[test]
fn a_golden_that_is_not_text_is_reported_as_unreadable() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::with_recorded("")?;
    fs::write(fixture.path(), [0xff, 0xfe])?;
    assert!(matches!(fixture.golden().verify_in(Mode::Check, ""), Err(Failure::Unreadable { .. })));
    Ok(())
}

#[test]
fn recording_is_chosen_only_by_the_exact_value_outside_ci() {
    let on = Some(OsStr::new("1"));
    let ci = Some(OsStr::new("true"));
    let chosen = |record, ci| Mode::chosen(record, ci).ok();
    assert_eq!(chosen(None, None), Some(Mode::Check));
    assert_eq!(chosen(None, ci), Some(Mode::Check));
    assert_eq!(chosen(on, None), Some(Mode::Record));
    assert_eq!(chosen(on, ci), None);
    assert_eq!(chosen(Some(OsStr::new("0")), None), None);
    assert_eq!(chosen(Some(OsStr::new("")), None), None);
}

#[test]
fn a_returned_failure_prints_the_readable_message() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::with_recorded("old\n")?;
    let printed = fixture.golden().verify_in(Mode::Check, "new\n").map_err(|failure| format!("{failure:?}"));
    let wanted = format!(
        "{}: output differs from the golden\n@ first change at line 1; - golden, + actual\n- \"old\\n\"\n+ \"new\\n\"\n\nif this change is intended, run `GOLDEN_RECORD=1 cargo test` and review the diff",
        fixture.path().display()
    );
    assert_eq!(printed, Err(wanted));
    Ok(())
}

#[test]
fn a_golden_in_a_package_lives_under_its_goldens_directory() {
    assert_eq!(Golden::in_package("/crate", "a.txt"), Golden::located_at("/crate/tests/goldens/a.txt"));
}

#[test]
fn equal_texts_have_no_difference() {
    assert_eq!(Difference::between("a\nb\n", "a\nb\n"), None);
}

#[test]
fn the_golden_macro_finds_a_golden_in_the_calling_package() -> Result<(), Failure> {
    golden!("macro.txt", "found through the macro\n")
}
