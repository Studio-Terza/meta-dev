use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use mutation_report::Cli;
use serde_json::{Value, json};

const GOLDEN_SOURCE: &str = "\
#[test]
fn report_is_stable() -> Result<(), golden::Failure> {
    golden::golden!(\"report.txt\", &render())
}

#[test]
fn spaced() {
    golden!  (\"x\")
}

#[test]
fn uses_type() {
    let found = Golden::in_running_package(\"x\");
}

#[test]
fn uses_method() {
    thing.golden(1);
}

#[test]
fn plain() {
    assert!(true);
}

#[test]
#[ignore]
fn later_attribute_hides_the_body() {
    golden!(\"y\");
}

#[tokio::test]
async fn awaited() {
    golden!(\"z\");
}

#[test]
fn  wide_spacing()  {
    golden!(\"w\");
}

#[test]
fn macro_without_call() {
    let golden = 1;
}

#[test]
fn mention_without_call() {
    golden! nothing
}

#[test]
fnord() {
    golden!(\"q\");
}

#[test]
fn unicode_ünder_score() {
    golden!(\"u\");
}
";
const REPORT: &str = "\
## Mutation matrix

5 mutants, 4 tests, 7 golden tests.

### Surviving mutants (2)

- a-missed
- m-missed

### Timed out mutants (1)

- m-timeout

### Tests that kill no mutant (2)

- b1 :: unused
- b2 :: other

### Mutants killed while no golden test fails (1)

- m-killed-nongolden -> b1 :: plain
";
const EMPTY_REPORT: &str = "\
## Mutation matrix

0 mutants, 0 tests, 0 golden tests.

### Surviving mutants (0)

none

### Timed out mutants (0)

none

### Tests that kill no mutant (0)

none

### Mutants killed while no golden test fails (0)

none
";

struct Checkout(tempfile::TempDir);

impl Checkout {
    fn new() -> Result<Checkout, io::Error> {
        let checkout = Checkout(tempfile::tempdir()?);
        checkout.write("sources/crates/foo/tests/golden.rs", GOLDEN_SOURCE)?;
        checkout.write("sources/crates/foo/notes.txt", "#[test]\nfn hidden() { Golden::x }\n")?;
        fs::create_dir(checkout.path("shards"))?;
        Ok(checkout)
    }

    fn path(&self, name: &str) -> PathBuf {
        self.0.path().join(name)
    }

    fn write(&self, name: &str, text: impl AsRef<[u8]>) -> Result<(), io::Error> {
        let file = self.path(name);
        fs::create_dir_all(file.parent().unwrap_or(Path::new("")))?;
        fs::write(file, text)
    }

    fn shard(&self, name: &str, matrix: &Value) -> Result<(), io::Error> {
        self.write(&format!("shards/{name}/matrix.json"), matrix.to_string())
    }

    fn report(&self) -> Answer {
        self.report_of("shards", "sources")
    }

    fn report_of(&self, shards: &str, sources: &str) -> Answer {
        Answer::to(&[
            self.path(shards).as_os_str(),
            self.path(sources).as_os_str(),
            self.path("report.md").as_os_str(),
            self.path("report.json").as_os_str(),
        ])
    }

    fn markdown(&self) -> String {
        fs::read_to_string(self.path("report.md")).unwrap_or_default()
    }

    fn field(&self, name: &str) -> Value {
        self.json().get(name).cloned().unwrap_or_default()
    }

    fn json(&self) -> Value {
        fs::read_to_string(self.path("report.json"))
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }
}

#[derive(Debug, PartialEq)]
struct Answer {
    exit: ExitCode,
    err: String,
}

impl Answer {
    fn to(arguments: &[&OsStr]) -> Answer {
        let mut err = Vec::new();
        let exit = Cli::answer(arguments.iter().map(OsString::from), &mut err);
        Answer { exit, err: String::from_utf8(err).unwrap_or_default() }
    }
}

struct Shards;

impl Shards {
    fn mutant(name: &str, summary: &str, failed: &[&str]) -> Value {
        json!({ "name": name, "summary": summary, "failed": failed })
    }

    fn two(checkout: &Checkout) -> Result<(), io::Error> {
        checkout.shard(
            "shard-0",
            &json!({
                "baseline": ["b1 :: report_is_stable", "b1 :: plain", "b1 :: unused"],
                "mutants": [
                    Self::mutant("m-missed", "MissedMutant", &[]),
                    Self::mutant("m-timeout", "Timeout", &[]),
                    Self::mutant("m-killed-golden", "CaughtMutant", &["b1 :: report_is_stable", "b1 :: plain"]),
                    Self::mutant("m-killed-nongolden", "CaughtMutant", &["b1 :: plain"]),
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
fn the_report_lists_survivors_idle_tests_and_kills_that_no_golden_saw() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    Shards::two(&checkout)?;
    assert_eq!(checkout.report(), Answer { exit: ExitCode::SUCCESS, err: String::new() });
    assert_eq!(checkout.markdown(), REPORT);
    assert_eq!(
        checkout.json(),
        json!({
            "mutants": 5,
            "tests": 4,
            "golden_tests": 7,
            "surviving": ["a-missed", "m-missed"],
            "timed_out": ["m-timeout"],
            "tests_that_kill_nothing": ["b1 :: unused", "b2 :: other"],
            "killed_with_no_golden_failing": ["m-killed-nongolden -> b1 :: plain"],
        })
    );
    Ok(())
}

#[test]
fn no_shards_gives_an_empty_report() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    checkout.write("shards/not-a-shard/readme.txt", "x")?;
    checkout.write("shards/loose-file.json", "{}")?;
    checkout.write("bare/crates/empty/src/lib.rs", "")?;
    assert_eq!(checkout.report_of("shards", "bare").exit, ExitCode::SUCCESS);
    assert_eq!(checkout.markdown(), EMPTY_REPORT);
    assert_eq!(checkout.field("mutants"), json!(0));
    Ok(())
}

#[test]
fn a_source_file_with_a_byte_that_is_not_utf8_does_not_stop_the_report() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    checkout.write("sources/crates/foo/tests/crash/input.rs", b"\xff\xfe")?;
    checkout.shard("shard-0", &json!({ "baseline": [], "mutants": [] }))?;
    assert_eq!(checkout.report().exit, ExitCode::SUCCESS);
    assert_eq!(checkout.field("golden_tests"), json!(7));
    Ok(())
}

#[test]
fn a_test_name_is_golden_by_its_last_path_segment() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    checkout.shard(
        "shard-0",
        &json!({
            "baseline": [],
            "mutants": [
                Shards::mutant("path", "CaughtMutant", &["bin :: tests::report_is_stable"]),
                Shards::mutant("bare", "CaughtMutant", &["report_is_stable"]),
                Shards::mutant("spaced", "CaughtMutant", &["bin ::   spaced"]),
                Shards::mutant("other", "CaughtMutant", &["bin :: tests::other"]),
                Shards::mutant("first", "CaughtMutant", &["bin :: plain", "bin :: tests::other"]),
            ],
        }),
    )?;
    checkout.report();
    assert_eq!(
        checkout.field("killed_with_no_golden_failing"),
        json!(["first -> bin :: plain, bin :: tests::other", "other -> bin :: tests::other"])
    );
    Ok(())
}

#[test]
fn killed_mutants_are_listed_by_name_and_only_when_they_were_killed() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    checkout.shard(
        "shard-0",
        &json!({
            "baseline": [],
            "mutants": [
                Shards::mutant("b", "CaughtMutant", &["x :: t2"]),
                Shards::mutant("a", "CaughtMutant", &["x :: t1", "x :: t3"]),
                Shards::mutant("unkilled", "MissedMutant", &[]),
            ],
        }),
    )?;
    checkout.report();
    assert_eq!(checkout.field("killed_with_no_golden_failing"), json!(["a -> x :: t1, x :: t3", "b -> x :: t2"]));
    assert_eq!(checkout.field("surviving"), json!(["unkilled"]));
    Ok(())
}

#[test]
fn a_section_shows_300_bullets_and_counts_the_rest() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    let missed: Vec<Value> =
        (0..305).map(|index| Shards::mutant(&format!("m{index:03}"), "MissedMutant", &[])).collect();
    checkout.shard("shard-0", &json!({ "baseline": [], "mutants": missed }))?;
    checkout.report();
    let markdown = checkout.markdown();
    assert!(markdown.contains("### Surviving mutants (305)\n\n- m000\n- m001\n"));
    assert!(markdown.contains("- m299\n- ... 5 more in report.json\n"));
    assert!(!markdown.contains("- m300"));
    assert_eq!(checkout.field("surviving").as_array().map(Vec::len), Some(305));
    Ok(())
}

#[test]
fn exactly_300_items_are_all_shown_with_no_remainder() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    let missed: Vec<Value> =
        (0..300).map(|index| Shards::mutant(&format!("m{index:03}"), "MissedMutant", &[])).collect();
    checkout.shard("shard-0", &json!({ "baseline": [], "mutants": missed }))?;
    checkout.report();
    let markdown = checkout.markdown();
    assert!(markdown.contains("- m299\n\n### Timed out mutants (0)"));
    assert!(!markdown.contains("more in report.json"));
    Ok(())
}

#[test]
fn a_shard_matrix_that_cannot_be_parsed_is_refused_by_name() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    checkout.write("shards/shard-0/matrix.json", "not json")?;
    let answer = checkout.report();
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert!(answer.err.contains("cannot parse") && answer.err.contains("shard-0/matrix.json"));
    assert!(!checkout.path("report.md").exists());
    Ok(())
}

#[test]
fn shards_or_sources_that_cannot_be_read_are_refused() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    let answer = checkout.report_of("absent", "sources");
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert!(answer.err.contains("cannot use") && answer.err.contains("absent"));
    fs::create_dir(checkout.path("bare"))?;
    let answer = checkout.report_of("shards", "bare");
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert!(answer.err.contains("cannot use") && answer.err.contains("crates"));
    Ok(())
}

#[test]
fn a_report_that_cannot_be_written_is_refused() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    let answer = Answer::to(&[
        checkout.path("shards").as_os_str(),
        checkout.path("sources").as_os_str(),
        "/no/such/dir/report.md".as_ref(),
        checkout.path("report.json").as_os_str(),
    ]);
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert!(answer.err.contains("cannot use /no/such/dir/report.md"));
    let answer = Answer::to(&[
        checkout.path("shards").as_os_str(),
        checkout.path("sources").as_os_str(),
        checkout.path("report.md").as_os_str(),
        "/no/such/dir/report.json".as_ref(),
    ]);
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert!(answer.err.contains("cannot use /no/such/dir/report.json"));
    Ok(())
}

#[test]
fn the_wrong_number_of_arguments_is_refused_with_the_usage() {
    for arguments in [&[][..], &["a".as_ref(), "b".as_ref(), "c".as_ref()][..], &["a".as_ref(); 5][..]] {
        let answer = Answer::to(arguments);
        assert_eq!(answer.exit, ExitCode::FAILURE);
        assert_eq!(answer.err, "usage: mutation-report <shards> <sources> <report.md> <report.json>\n");
    }
}
