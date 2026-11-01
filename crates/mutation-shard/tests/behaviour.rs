use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use mutation_shard::Cli;
use serde_json::{Value, json};

const BASELINE_LOG: &str = "   Running unittests src/lib.rs (target/debug/deps/foo-1)\ntest a ... ok\ntest b ... ok\n   Doc-tests foo\ntest src/lib.rs - f (line 1) ... ok\n";
const KILLING_LOG: &str =
    "Running unittests src/lib.rs (target/debug/deps/foo-1)\ntest a ... FAILED\ntest b ... ok\ntest c ... FAILED\n";
const OTHER_KILLING_LOG: &str = "Running tests/t.rs (target/debug/deps/t-2)\ntest z ... FAILED\n";
const PASSING_LOG: &str = "Running unittests src/lib.rs (target/debug/deps/foo-1)\ntest a ... ok\n";

struct Mutants(tempfile::TempDir);

impl Mutants {
    fn new() -> Result<Mutants, io::Error> {
        let mutants = Mutants(tempfile::tempdir()?);
        fs::create_dir(mutants.0.path().join("log"))?;
        Ok(mutants)
    }

    fn log(&self, name: &str, text: &str) -> Result<(), io::Error> {
        fs::write(self.0.path().join("log").join(name), text)
    }

    fn outcomes(&self, outcomes: &Value) -> Result<(), io::Error> {
        fs::write(self.0.path().join("outcomes.json"), json!({ "outcomes": outcomes }).to_string())
    }

    fn path(&self) -> &Path {
        self.0.path()
    }

    fn destination(&self) -> PathBuf {
        self.0.path().join("matrix.json")
    }

    fn shard(&self) -> Answer {
        Answer::to(&[self.path().as_os_str(), self.destination().as_os_str()])
    }

    fn mutants_field(&self, name: &str) -> Vec<Value> {
        self.matrix()
            .get("mutants")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .map(|mutant| mutant.get(name).cloned().unwrap_or_default())
            .collect()
    }

    fn baseline_field(&self) -> Value {
        self.matrix().get("baseline").cloned().unwrap_or_default()
    }

    fn matrix(&self) -> Value {
        fs::read_to_string(self.destination())
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

struct Outcome;

impl Outcome {
    fn baseline() -> Value {
        json!({ "scenario": "Baseline", "summary": "Success", "log_path": "log/baseline.log", "phase_results": [] })
    }

    fn mutant(name: impl Into<Value>, summary: &str, log: impl Into<Value>, phases: &Value) -> Value {
        json!({ "scenario": { "Mutant": name.into() }, "summary": summary, "log_path": log.into(), "phase_results": phases })
    }
}

#[test]
fn a_shard_lists_the_baseline_tests_and_the_tests_that_failed_per_mutant() -> Result<(), Box<dyn Error>> {
    let mutants = Mutants::new()?;
    mutants.log("baseline.log", BASELINE_LOG)?;
    mutants.log("one.log", KILLING_LOG)?;
    mutants.log("two.log", PASSING_LOG)?;
    mutants.outcomes(&json!([
        Outcome::baseline(),
        Outcome::mutant(
            json!({ "name": "src/a.rs: replace f with 0" }),
            "CaughtMutant",
            json!("log/one.log"),
            &json!([])
        ),
        Outcome::mutant(
            json!({ "name": "src/b.rs: replace g with 1" }),
            "MissedMutant",
            json!("log/two.log"),
            &json!([])
        ),
    ]))?;
    assert_eq!(mutants.shard(), Answer { exit: ExitCode::SUCCESS, err: String::new() });
    assert_eq!(
        mutants.matrix(),
        json!({
            "baseline": ["doc-foo :: src/lib.rs - f (line 1)", "foo-1 :: a", "foo-1 :: b"],
            "mutants": [
                { "name": "src/a.rs: replace f with 0", "summary": "CaughtMutant", "failed": ["foo-1 :: a", "foo-1 :: c"] },
                { "name": "src/b.rs: replace g with 1", "summary": "MissedMutant", "failed": [] },
            ],
        })
    );
    Ok(())
}

#[test]
fn a_mutant_without_a_name_is_titled_by_its_file_and_replacement() -> Result<(), Box<dyn Error>> {
    let mutants = Mutants::new()?;
    mutants.outcomes(&json!([
        Outcome::baseline(),
        Outcome::mutant(json!({ "file": "src/a.rs", "replacement": "true" }), "Unviable", json!(null), &json!([])),
        Outcome::mutant(
            json!({ "name": "", "file": "src/b.rs", "replacement": "false" }),
            "Unviable",
            json!(null),
            &json!([])
        ),
        Outcome::mutant(json!({ "replacement": "x" }), "Unviable", json!(null), &json!([])),
        Outcome::mutant(json!({ "file": "src/c.rs" }), "Unviable", json!(null), &json!([])),
        Outcome::mutant(json!({}), "Unviable", json!(null), &json!([])),
    ]))?;
    mutants.shard();
    let titles = mutants.mutants_field("name");
    assert_eq!(titles, ["src/a.rs: true", "src/b.rs: false", "?: x", "src/c.rs: ", "?: "]);
    Ok(())
}

#[test]
fn the_log_is_the_first_one_that_exists_the_mutant_log_before_its_phases() -> Result<(), Box<dyn Error>> {
    let mutants = Mutants::new()?;
    mutants.log("baseline.log", BASELINE_LOG)?;
    mutants.log("main.log", KILLING_LOG)?;
    mutants.log("build.log", OTHER_KILLING_LOG)?;
    mutants.log("test.log", PASSING_LOG)?;
    let phases = json!([{ "log_path": null }, { "log_path": "log/absent.log" }, { "log_path": "log/build.log" }, { "log_path": "log/test.log" }]);
    mutants.outcomes(&json!([
        Outcome::baseline(),
        Outcome::mutant(json!({ "name": "main" }), "CaughtMutant", json!("log/main.log"), &phases),
        Outcome::mutant(json!({ "name": "phase" }), "CaughtMutant", json!("log/absent.log"), &phases),
        Outcome::mutant(json!({ "name": "phase-only" }), "CaughtMutant", json!(null), &phases),
        Outcome::mutant(
            json!({ "name": "no-log" }),
            "CaughtMutant",
            json!("log/absent.log"),
            &json!([{ "log_path": "log/none.log" }])
        ),
    ]))?;
    mutants.shard();
    let failed = mutants.mutants_field("failed");
    assert_eq!(failed, [json!(["foo-1 :: a", "foo-1 :: c"]), json!(["t-2 :: z"]), json!(["t-2 :: z"]), json!([])]);
    Ok(())
}

#[test]
fn a_directory_in_place_of_a_log_is_no_log() -> Result<(), Box<dyn Error>> {
    let mutants = Mutants::new()?;
    fs::create_dir(mutants.path().join("log").join("dir.log"))?;
    mutants.log("baseline.log", BASELINE_LOG)?;
    mutants.outcomes(&json!([
        Outcome::baseline(),
        Outcome::mutant(json!({ "name": "d" }), "CaughtMutant", json!("log/dir.log"), &json!([])),
    ]))?;
    assert_eq!(mutants.shard().exit, ExitCode::SUCCESS);
    assert_eq!(mutants.mutants_field("failed"), [json!([])]);
    Ok(())
}

#[test]
fn only_the_baseline_scenario_gives_baseline_tests() -> Result<(), Box<dyn Error>> {
    let mutants = Mutants::new()?;
    mutants.log("baseline.log", BASELINE_LOG)?;
    mutants.log("one.log", KILLING_LOG)?;
    mutants.outcomes(&json!([
        Outcome::mutant(json!({ "name": "m" }), "CaughtMutant", json!("log/one.log"), &json!([])),
        Outcome::baseline(),
    ]))?;
    mutants.shard();
    assert_eq!(mutants.baseline_field(), json!(["doc-foo :: src/lib.rs - f (line 1)", "foo-1 :: a", "foo-1 :: b"]));
    assert_eq!(mutants.mutants_field("name").len(), 1);
    Ok(())
}

#[test]
fn outcomes_that_are_missing_or_broken_are_refused() -> Result<(), Box<dyn Error>> {
    let mutants = Mutants::new()?;
    let answer = mutants.shard();
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert!(answer.err.contains("cannot use") && answer.err.contains("outcomes.json"));
    fs::write(mutants.path().join("outcomes.json"), "not json")?;
    let answer = mutants.shard();
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert!(answer.err.contains("cannot parse") && answer.err.contains("outcomes.json"));
    assert!(!mutants.destination().exists());
    Ok(())
}

#[test]
fn a_matrix_that_cannot_be_written_is_refused() -> Result<(), Box<dyn Error>> {
    let mutants = Mutants::new()?;
    mutants.outcomes(&json!([Outcome::baseline()]))?;
    let answer = Answer::to(&[mutants.path().as_os_str(), "/no/such/dir/matrix.json".as_ref()]);
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert!(answer.err.contains("cannot use /no/such/dir/matrix.json"));
    Ok(())
}

#[test]
fn the_wrong_number_of_arguments_is_refused_with_the_usage() {
    for arguments in [&[][..], &["a".as_ref()][..], &["a".as_ref(), "b".as_ref(), "c".as_ref()][..]] {
        let answer = Answer::to(arguments);
        assert_eq!(answer.exit, ExitCode::FAILURE);
        assert_eq!(answer.err, "usage: mutation-shard <mutants.out> <out.json>\n");
    }
}
