mod ignored;

use std::error::Error;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::ExitCode;

use console_ports::Complaint;
use file_ports::{Bytes, FileFailure, Files, Kind, Memory};
use ignored::Ignored;
use invariants::Ignoring;
use mutation_report::Cli;
use serde_json::{Value, json};

const OVERFLOWING_SECTION: usize = 305;
const FULL_SECTION: usize = 300;

#[derive(Debug, PartialEq)]
struct Answer {
    exit: ExitCode,
    err: String,
}

impl Answer {
    fn to(files: &impl Files, arguments: &[&OsStr]) -> Answer {
        let answered = Cli::answer(arguments, files);
        let err: &Complaint = answered.as_ref();
        Answer { exit: answered.code(), err: AsRef::<str>::as_ref(err).to_owned() }
    }

    fn usage() -> Answer {
        Answer { exit: ExitCode::FAILURE, err: format!("{USAGE}\n") }
    }
}

struct Shards;

impl Shards {
    fn mutant(name: &str, summary: &str, failed: &[&str]) -> Value {
        json!({ "name": name, "summary": summary, "failed": failed })
    }

    fn two(checkout: &Checkout) -> Result<(), FileFailure> {
        checkout.shard(
            "shard-0",
            &json!({
                "baseline": ["b1 :: report_is_stable", "b1 :: plain", "b1 :: unused"],
                "mutants": [
                    Self::mutant("m-missed", "MissedMutant", &[]),
                    Self::mutant("m-timeout", "Timeout", &[]),
                    Self::mutant("m-killed", "CaughtMutant", &["b1 :: report_is_stable", "b1 :: plain"]),
                    Self::mutant("m-killed-once", "CaughtMutant", &["b1 :: plain"]),
                ],
            }),
        )?;
        checkout.shard(
            "shard-1",
            &json!({ "baseline": ["b1 :: plain", "b2 :: other"], "mutants": [Self::mutant("a-missed", "MissedMutant", &[])] }),
        )
    }
}

#[test]
fn the_report_lists_survivors_timeouts_and_idle_tests() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    Shards::two(&checkout)?;
    assert_eq!(checkout.report(), Answer { exit: ExitCode::SUCCESS, err: String::new() });
    assert_eq!(
        checkout.markdown(),
        "## Mutation matrix\n\n5 mutants, 4 tests.\n\n### Surviving mutants (2)\n\n- a-missed\n- m-missed\n\n\
         ### Timed out mutants (1)\n\n- m-timeout\n\n### Tests that kill no mutant (2)\n\n- b1 :: unused\n- b2 :: other\n\n\
         ### Unviable mutants by crate (1)\n\n- all crates: 0 of 5 unviable\n"
    );
    assert_eq!(
        checkout.json(),
        json!({
            "mutants": 5_usize,
            "tests": 4_usize,
            "surviving": ["a-missed", "m-missed"],
            "timed_out": ["m-timeout"],
            "tests_that_kill_nothing": ["b1 :: unused", "b2 :: other"],
            "unviable": ["all crates: 0 of 5 unviable"],
        })
    );
    Ok(())
}

#[test]
fn no_shards_gives_an_empty_report() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    checkout.write("shards/not-a-shard/readme.txt", "x")?;
    checkout.write("shards/loose-file.json", "{}")?;
    assert_eq!(checkout.report().exit, ExitCode::SUCCESS);
    assert_eq!(
        checkout.markdown(),
        "## Mutation matrix\n\n0 mutants, 0 tests.\n\n### Surviving mutants (0)\n\nnone\n\n\
         ### Timed out mutants (0)\n\nnone\n\n### Tests that kill no mutant (0)\n\nnone\n\n\
         ### Unviable mutants by crate (1)\n\n- all crates: 0 of 0 unviable\n"
    );
    assert_eq!(checkout.field("mutants"), json!(0_usize));
    Ok(())
}

#[test]
fn a_section_shows_300_bullets_and_counts_the_rest() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    let missed: Vec<Value> =
        (0..OVERFLOWING_SECTION).map(|index| Shards::mutant(&format!("m{index:03}"), "MissedMutant", &[])).collect();
    checkout.shard("shard-0", &json!({ "baseline": [], "mutants": missed }))?;
    checkout.report();
    let markdown = checkout.markdown();
    assert!(markdown.contains("### Surviving mutants (305)\n\n- m000\n- m001\n"));
    assert!(markdown.contains("- m299\n- ... 5 more in report.json\n"));
    assert_eq!(markdown.matches("- m300").count(), 0);
    assert_eq!(checkout.field("surviving").as_array().map(Vec::len), Some(OVERFLOWING_SECTION));
    Ok(())
}

#[test]
fn exactly_300_items_are_all_shown_with_no_remainder() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    let missed: Vec<Value> =
        (0..FULL_SECTION).map(|index| Shards::mutant(&format!("m{index:03}"), "MissedMutant", &[])).collect();
    checkout.shard("shard-0", &json!({ "baseline": [], "mutants": missed }))?;
    checkout.report();
    let markdown = checkout.markdown();
    assert!(markdown.contains("- m299\n\n### Timed out mutants (0)"));
    assert_eq!(markdown.matches("more in report.json").count(), 0);
    Ok(())
}

#[test]
fn a_shard_matrix_that_cannot_be_parsed_is_refused_by_name() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    checkout.write("shards/shard-0/matrix.json", "not json")?;
    let answer = checkout.report();
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert!(answer.err.contains("cannot parse"));
    assert!(answer.err.contains("shard-0/matrix.json"));
    assert_eq!(checkout.files.kind(&checkout.path("report.md")), Kind::Missing);
    Ok(())
}

#[test]
fn shards_that_cannot_be_read_are_refused() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    let absent = checkout.report_of("absent");
    assert_eq!(absent.exit, ExitCode::FAILURE);
    assert!(absent.err.contains("cannot use"));
    assert!(absent.err.contains("absent"));
    Ok(())
}

#[test]
fn a_report_that_cannot_be_written_is_refused() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    let unwritable_markdown = Answer::to(
        &checkout.files,
        &[
            checkout.path("shards").as_os_str(),
            "/no/such/dir/report.md".as_ref(),
            checkout.path("report.json").as_os_str(),
        ],
    );
    assert_eq!(unwritable_markdown.exit, ExitCode::FAILURE);
    assert!(unwritable_markdown.err.contains("cannot use /no/such/dir/report.md"));
    let unwritable_json = Answer::to(
        &checkout.files,
        &[
            checkout.path("shards").as_os_str(),
            checkout.path("report.md").as_os_str(),
            "/no/such/dir/report.json".as_ref(),
        ],
    );
    assert_eq!(unwritable_json.exit, ExitCode::FAILURE);
    assert!(unwritable_json.err.contains("cannot use /no/such/dir/report.json"));
    Ok(())
}

const USAGE: &str = "usage: mutation-report <shards> <report.md> <report.json>";

#[test]
fn the_wrong_number_of_arguments_is_refused_with_the_usage() {
    let memory = Memory::default();
    assert_eq!(Answer::to(&memory, &[]), Answer::usage());
    assert_eq!(Answer::to(&memory, &[OsStr::new("a"), OsStr::new("b")]), Answer::usage());
    assert_eq!(Answer::to(&memory, &[OsStr::new("a"); 4]), Answer::usage());
}

struct Checkout {
    files: Memory,
    root: PathBuf,
}

impl Checkout {
    fn new() -> Result<Checkout, FileFailure> {
        let checkout = Checkout { files: Memory::default(), root: PathBuf::from("/checkout") };
        checkout.directory("shards")?;
        Ok(checkout)
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }

    fn write(&self, name: &str, text: impl AsRef<[u8]>) -> Result<(), FileFailure> {
        let file = self.path(name);
        file.parent().map_or(Ok(()), |parent| self.files.create_directories(parent))?;
        self.files.write(&file, Bytes::new(text.as_ref().to_vec()))
    }

    fn directory(&self, name: &str) -> Result<(), FileFailure> {
        self.files.create_directories(&self.path(name))
    }

    fn shard(&self, name: &str, matrix: &Value) -> Result<(), FileFailure> {
        self.write(&format!("shards/{name}/matrix.json"), matrix.to_string())
    }

    fn report(&self) -> Answer {
        self.report_of("shards")
    }

    fn report_of(&self, shards: &str) -> Answer {
        Answer::to(
            &self.files,
            &[self.path(shards).as_os_str(), self.path("report.md").as_os_str(), self.path("report.json").as_os_str()],
        )
    }

    fn text(&self, name: &str) -> Option<String> {
        self.files.read_text(&self.path(name)).ignoring(Ignored::Unread).map(|text| text.to_string())
    }

    fn markdown(&self) -> String {
        self.text("report.md").unwrap_or_default()
    }

    fn field(&self, name: &str) -> Value {
        self.json().get(name).cloned().unwrap_or_default()
    }

    fn json(&self) -> Value {
        self.text("report.json")
            .and_then(|text| serde_json::from_str(&text).ignoring(Ignored::Unread))
            .unwrap_or_default()
    }
}

#[test]
fn unviable_mutants_are_counted_for_all_crates_and_for_each_crate() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    checkout.shard(
        "shard-0",
        &json!({
            "baseline": ["b1 :: plain"],
            "mutants": [
                Shards::mutant("crates/a/src/lib.rs:1:1: replace f with g", "Unviable", &[]),
                Shards::mutant("crates/a/src/lib.rs:2:1: replace h with i", "CaughtMutant", &["b1 :: plain"]),
                Shards::mutant("crates/b/src/lib.rs:1:1: replace j with k", "Unviable", &[]),
            ],
        }),
    )?;
    assert_eq!(checkout.report().exit, ExitCode::SUCCESS);
    assert_eq!(
        checkout.field("unviable"),
        json!(["all crates: 2 of 3 unviable", "a: 1 of 2 unviable", "b: 1 of 1 unviable"])
    );
    Ok(())
}
