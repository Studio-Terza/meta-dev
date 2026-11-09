mod differences;

use std::ffi::{OsStr, OsString};

use console_ports::Variables;
use file_ports::{Files, Kind};
use fixture::Fixture;
use golden::{Difference, Failure, Golden, Mode, Output, golden};
use laws::proptest;
use laws::{roundtrip, table};
use proptest::prelude::any;

#[test]
fn checking_matching_output_passes() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::with_recorded("same\n")?;
    Fixture::golden().verify(&Variables::default(), fixture.files(), Output::new("same\n"))?;
    Ok(())
}

#[test]
fn a_check_fails_on_changed_output_and_leaves_the_golden_alone() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::with_recorded("old\n")?;
    assert!(matches!(
        Fixture::golden().verify(&Variables::default(), fixture.files(), Output::new("new\n")),
        Err(Failure::Differs { .. })
    ));
    assert!(Fixture::golden().verify(&Variables::default(), fixture.files(), Output::new("old\n")).is_ok());
    Ok(())
}

#[test]
fn a_check_never_creates_a_missing_golden() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    assert!(matches!(
        Fixture::golden().verify(&Variables::default(), fixture.files(), Output::new("new\n")),
        Err(Failure::Missing { .. })
    ));
    assert_eq!(fixture.files().kind(&Fixture::path()), Kind::Missing);
    Ok(())
}

#[test]
fn a_golden_that_is_not_text_is_reported_as_unreadable() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::with_recorded("")?;
    fixture.overwrite(&[0xff, 0xfe])?;
    assert!(matches!(
        Fixture::golden().verify(&Variables::default(), fixture.files(), Output::new("")),
        Err(Failure::Unreadable { .. })
    ));
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
    |text: &String| Fixture::recorded_through_the_api(text),
    |fixture: &Option<Fixture>| fixture.as_ref()?.text().ok(),
    Some
);

#[test]
fn a_returned_failure_prints_the_readable_message() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::with_recorded("old\n")?;
    let printed = Fixture::golden()
        .verify(&Variables::default(), fixture.files(), Output::new("new\n"))
        .err()
        .map(|failure| format!("{failure:?}"));
    let heading = format!("{}: output differs from the golden", Fixture::path().display());
    let lines =
        printed.as_deref().map(|text| text.parse::<text_lines::Lines>().into_iter().flatten().collect::<Vec<String>>());
    let wanted = [
        heading.as_str(),
        "@ first change at line 1; - golden, + actual",
        "- \"old\\n\"",
        "+ \"new\\n\"",
        "",
        "if this change is intended, run `GOLDEN_RECORD=1 cargo test` and review the diff",
    ];
    assert_eq!(lines, Some(wanted.map(str::to_owned).to_vec()));
    Ok(())
}

#[test]
fn a_test_cannot_choose_to_record_a_package_golden() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::with_recorded("old")?;
    fixture.mark_package()?;
    let recorded =
        Fixture::golden().verify_in(&Variables::default(), fixture.files(), Mode::Record, Output::new("new"));
    assert!(matches!(recorded, Err(Failure::NotChosen { .. })));
    assert_eq!(fixture.text()?, "old");
    Ok(())
}

#[test]
fn recording_chosen_by_the_environment_rewrites_a_package_golden() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::with_recorded("old")?;
    fixture.mark_package()?;
    let chosen: Variables = [(OsString::from(Mode::VARIABLE), OsString::from(Mode::ON))].into_iter().collect();
    Fixture::golden().verify_in(&chosen, fixture.files(), Mode::Record, Output::new("new"))?;
    assert_eq!(fixture.text()?, "new");
    Ok(())
}

#[test]
fn a_golden_in_a_package_lives_under_its_goldens_directory() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    Fixture::golden().verify_in(&Variables::default(), fixture.files(), Mode::Record, Output::new("placed\n"))?;
    assert!(Fixture::golden().verify(&Variables::default(), fixture.files(), Output::new("placed\n")).is_ok());
    Ok(())
}

table!(
    a_name_that_leaves_the_goldens_directory_is_refused,
    |name: &&str| matches!(Golden::in_package("/crate", name), Err(Failure::Escapes { .. })),
    [
        "../snap/a.txt" => true,
        "nested/../../a.txt" => true,
        "/etc/a.txt" => true,
        "nested/a.txt" => false,
        "a.txt" => false,
    ]
);

#[test]
fn equal_texts_have_no_difference() {
    assert_eq!(Difference::between(Output::new("a\nb\n"), Output::new("a\nb\n")), None);
}

#[test]
fn the_golden_macro_finds_a_golden_in_the_calling_package() -> Result<(), Failure> {
    golden!("macro.txt", "found through the macro\n")
}

#[test]
fn the_fixtures_macro_reads_a_tree_of_the_calling_package() -> Result<(), Box<dyn std::error::Error>> {
    let goldens = golden::fixtures!("tests/goldens")?;
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/goldens");
    assert_eq!(goldens.kind(&directory.join("macro.txt")), Kind::File);
    assert_eq!(goldens.kind(&directory.join("changed_line.diff")), Kind::File);
    Ok(())
}

mod fixture {
    use std::path::{Path, PathBuf};

    use console_ports::Variables;
    use file_ports::{Bytes, FileFailure, Files, Memory};
    use golden::{Golden, Mode, Output};
    use newtype::Newtype;

    #[derive(Default)]
    pub struct Fixture {
        files: Memory,
    }

    impl Fixture {
        pub const NAME: &str = "nested/output.txt";
        pub const GOLDENS: &str = "tests/goldens";
        pub const PACKAGE: &str = "/package";

        pub fn new() -> Result<Self, FileFailure> {
            let fixture = Self::default();
            fixture.files.create_directories(Path::new(Self::PACKAGE))?;
            Ok(fixture)
        }

        pub fn files(&self) -> &Memory {
            &self.files
        }

        pub fn path() -> PathBuf {
            Path::new(Self::PACKAGE).join(Self::GOLDENS).join(Self::NAME)
        }

        pub fn golden() -> Golden {
            match Golden::in_package(Self::PACKAGE, Self::NAME) {
                Ok(golden) => golden,
                Err(failure) => unreachable!("the fixture name stays inside the goldens directory: {failure:?}"),
            }
        }

        pub fn with_recorded(text: &str) -> Result<Self, FileFailure> {
            let fixture = Self::new()?;
            fixture.files.create_directories(&Path::new(Self::PACKAGE).join(Self::GOLDENS).join("nested"))?;
            fixture.files.write(&Self::path(), Bytes::from(text))?;
            Ok(fixture)
        }

        pub fn overwrite(&self, bytes: &[u8]) -> Result<(), FileFailure> {
            self.files.write(&Self::path(), Bytes::new(bytes.to_vec()))
        }

        pub fn mark_package(&self) -> Result<(), FileFailure> {
            self.files.write(&Path::new(Self::PACKAGE).join("Cargo.toml"), Bytes::from("[package]\nname = \"a\"\n"))
        }

        pub fn text(&self) -> Result<String, FileFailure> {
            self.files.read_text(&Self::path()).map(Newtype::into_inner)
        }

        pub fn recorded_through_the_api(text: &str) -> Option<Self> {
            let fixture = Self::new().ok()?;
            Self::golden().verify_in(&Variables::default(), &fixture.files, Mode::Record, Output::new(text)).ok()?;
            Some(fixture)
        }
    }
}
