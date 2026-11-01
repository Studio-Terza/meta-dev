use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

use golden::{Difference, Failure, Golden, Mode, golden};
use laws::proptest;
use laws::{roundtrip, table};
use proptest::prelude::any;
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
        fs::create_dir_all(fixture.directory.path().join("nested"))?;
        fs::write(fixture.path(), text)?;
        Ok(fixture)
    }

    fn text(&self) -> Result<String, std::io::Error> {
        fs::read_to_string(self.path())
    }

    fn holding(text: &str) -> Option<Self> {
        Self::with_recorded(text).ok()
    }
}

#[test]
fn checking_matching_output_passes() -> Result<(), Box<dyn std::error::Error>> {
    Fixture::with_recorded("same\n")?.golden().verify("same\n")?;
    Ok(())
}

#[test]
fn a_check_fails_on_changed_output_and_leaves_the_golden_alone() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::with_recorded("old\n")?;
    assert!(matches!(fixture.golden().verify("new\n"), Err(Failure::Differs { .. })));
    assert_eq!(fixture.text()?, "old\n");
    Ok(())
}

#[test]
fn a_check_never_creates_a_missing_golden() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    assert!(matches!(fixture.golden().verify("new\n"), Err(Failure::Missing { .. })));
    assert!(!fixture.path().exists());
    Ok(())
}

#[test]
fn a_golden_that_is_not_text_is_reported_as_unreadable() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::with_recorded("")?;
    fs::write(fixture.path(), [0xff, 0xfe])?;
    assert!(matches!(fixture.golden().verify(""), Err(Failure::Unreadable { .. })));
    Ok(())
}

table!(
    recording_is_chosen_only_by_the_exact_value_outside_ci,
    |(record, ci): &(Option<&str>, Option<&str>)| Mode::chosen(record.map(OsStr::new), ci.map(OsStr::new)).ok(),
    [
        (None, None) => Some(Mode::Check),
        (None, Some("true")) => Some(Mode::Check),
        (Some("1"), None) => Some(Mode::Record),
        (Some("1"), Some("true")) => None,
        (Some("0"), None) => None,
        (Some(""), None) => None,
    ]
);

roundtrip!(
    any_text_is_recorded_exactly,
    any::<String>(),
    Fixture::holding,
    |fixture: &Option<Fixture>| fixture.as_ref()?.text().ok(),
    Some
);

#[test]
fn a_returned_failure_prints_the_readable_message() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::with_recorded("old\n")?;
    let printed = fixture.golden().verify("new\n").map_err(|failure| format!("{failure:?}"));
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
