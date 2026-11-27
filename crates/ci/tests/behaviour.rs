use std::fs;

use tempfile::TempDir;
use test_failure::Failed;

const LIMITS: &str = "limits.toml";
const TIMED: [&str; 3] = ["timed", LIMITS, "lint"];
const PLAN: [&str; 5] = ["mutants-plan", "listed.json", "reused.json", "caught.txt", "output"];
const PRIOR: [&str; 2] = ["numstat", "outcomes.json"];
const LISTED: &str = r#"[{"name":"a.rs:1:1: x","file":"a.rs"},{"name":"b.rs:1:1: y","file":"b.rs"},{"name":"b.rs:2:1: z","file":"b.rs"},{"name":"c.rs:1:1: v","file":"c.rs"},{"name":"d.rs:1:1: n","file":"d.rs"}]"#;
const OUTCOMES: &str = r#"{"outcomes":[{"scenario":"Baseline","summary":"Success"},{"scenario":{"Mutant":{"name":"a.rs:1:1: x","file":"a.rs"}},"summary":"CaughtMutant"},{"scenario":{"Mutant":{"name":"b.rs:1:1: y","file":"b.rs"}},"summary":"CaughtMutant"},{"scenario":{"Mutant":{"name":"b.rs:2:1: z","file":"b.rs"}},"summary":"MissedMutant"},{"scenario":{"Mutant":{"name":"c.rs:1:1: v","file":"c.rs"}},"summary":"Unviable"}]}"#;
const REUSED: &str = r#"{"outcomes":[{"scenario":{"Mutant":{"name":"b.rs:1:1: y","file":"b.rs"}},"summary":"CaughtMutant"},{"scenario":{"Mutant":{"name":"c.rs:1:1: v","file":"c.rs"}},"summary":"CaughtMutant"}]}"#;
const NONE: &str = r#"{"outcomes":[]}"#;
const FRESH: &str = "mutants: no earlier run to reuse";

#[derive(Debug)]
struct Case {
    arguments: Vec<&'static str>,
    files: Vec<File>,
    code: i32,
    out: String,
    err: &'static str,
    written: Vec<File>,
}

#[derive(Debug)]
struct File {
    path: String,
    text: String,
}

mod ports {
    use std::io;
    use std::path::Path;
    use std::process::{Command, Output};

    #[derive(Debug)]
    pub(crate) struct Binary;

    impl Binary {
        pub(crate) fn ran(directory: &Path, arguments: &[&str]) -> io::Result<Output> {
            Command::new(env!("CARGO_BIN_EXE_ci")).args(arguments).current_dir(directory).output()
        }
    }
}

#[test]
fn every_case_exits_prints_and_writes_as_expected() -> Result<(), Failed> {
    let drifted: Vec<Option<String>> = Case::all().iter().map(Case::drifted).collect::<Result<_, Failed>>()?;
    assert_eq!(drifted.into_iter().flatten().collect::<Vec<String>>(), Vec::<String>::new());
    Ok(())
}

impl Case {
    fn all() -> Vec<Case> {
        let many: Vec<String> =
            (0..121_u8).map(|line| format!(r#"{{"name":"m.rs:{line}:1: m","file":"m.rs"}}"#)).collect();
        let tally = |head: &str, listed: u8, reused: u8, shards: u8| {
            format!(
                "{head}\nmutants: {listed} in the diff, {reused} reused, {} to test in {shards} shards\n",
                listed - reused
            )
        };
        let failed = |err, listed, numstat| Case {
            code: 1,
            err,
            written: Vec::new(),
            ..Case::planned(listed, numstat, String::new(), ["", "", ""])
        };
        let zero = "warn = 0\nblock = 0";
        vec![
            Case::timed(&["true"], zero, 0, "lint: within warn 0 s and block 0 s, took 0.0 s\n", ""),
            Case::timed(
                &["sleep", "0.21"],
                "warn = 0\nblock = 1000",
                0,
                "::warning::lint: above warn 0 s, within block 1000 s, took 0.",
                "",
            ),
            Case::timed(&["sleep", "0.21"], zero, 1, "::error::lint: above block 0 s, took 0.", ""),
            Case::timed(
                &["false"],
                "warn = 0.5\nblock = 3.5",
                1,
                "lint: within warn 0.5 s and block 3.5 s, took 0.0 s\n::error::lint failed: exit status: 1\n",
                "",
            ),
            Case::timed(&["absent-program"], zero, 1, "", "cannot start absent-program: No such file or directory"),
            Case::timed(&["true"], "warn = -1\nblock = 0", 1, "", "a limit is not a number of seconds: "),
            Case::timed(&["true"], "warn = 0", 1, "", "cannot read the limits: "),
            Case {
                files: vec![File::at(LIMITS, "[clippy]\nwarn = 0\nblock = 0\n")],
                ..Case::timed(&["true"], "", 1, "", "the limits hold no stage named lint\n")
            },
            Case { files: Vec::new(), ..Case::timed(&["true"], "", 1, "", "cannot read limits.toml: ") },
            Case { arguments: vec!["timed"], ..Case::timed(&[], "", 1, "", "usage: ci timed ") },
            Case {
                arguments: PLAN.to_vec(),
                ..Case::planned(LISTED, "", tally(FRESH, 5, 0, 1), [NONE, "", "shards=[0]\n"])
            },
            Case::planned(
                LISTED,
                "1\t1\ta.rs\n3\t0\ttests/t.rs\n",
                tally("mutants: reusing the earlier run's 3 caught, outside the 2 files changed since", 5, 2, 1),
                [REUSED, "b.rs:1:1: y\nc.rs:1:1: v\n", "shards=[0]\n"],
            ),
            Case::planned(
                LISTED,
                "0\t2\ttests/t.rs\n",
                tally("mutants: reusing nothing, since test code lost lines in tests/t.rs", 5, 0, 1),
                [NONE, "", "shards=[0]\n"],
            ),
            Case {
                arguments: PLAN.to_vec(),
                ..Case::planned(
                    &format!("[{}]", many.join(",")),
                    "",
                    tally(FRESH, 121, 0, 3),
                    [NONE, "", "shards=[0,1,2]\n"],
                )
            },
            failed("a numstat row is not added, deleted and path separated by tabs: 1 a.rs\n", LISTED, "1 a.rs\n"),
            failed("cannot read the mutants: ", "{", ""),
            Case {
                arguments: PLAN.to_vec(),
                ..Case::planned("", "", tally(FRESH, 0, 0, 1), [NONE, "", "shards=[0]\n"])
            },
        ]
    }

    fn timed(command: &[&'static str], limits: &str, code: i32, out: &str, err: &'static str) -> Case {
        let arguments = TIMED.iter().chain(command).copied().collect();
        let files = vec![File::at(LIMITS, &format!("[lint]\n{limits}\n"))];
        Case { arguments, files, code, out: out.to_owned(), err, written: Vec::new() }
    }

    fn planned(listed: &str, numstat: &str, out: String, [reused, caught, shards]: [&str; 3]) -> Case {
        let files =
            [["listed.json", listed], ["numstat", numstat], ["outcomes.json", OUTCOMES], ["output", "earlier=1\n"]];
        let written = [["reused.json", reused], ["caught.txt", caught], ["output", &format!("earlier=1\n{shards}")]];
        let arguments = PLAN.iter().chain(&PRIOR).copied().collect();
        Case { arguments, files: File::all(files), code: 0, out, err: "", written: File::all(written) }
    }

    fn drifted(&self) -> Result<Option<String>, Failed> {
        let directory = TempDir::new()?;
        self.files.iter().try_for_each(|file| fs::write(directory.path().join(&file.path), &file.text))?;
        let ran = ports::Binary::ran(directory.path(), &self.arguments)?;
        let out = String::from_utf8(ran.stdout)?;
        let err = String::from_utf8(ran.stderr)?;
        let written: Vec<String> = self
            .written
            .iter()
            .map(|file| fs::read_to_string(directory.path().join(&file.path)))
            .collect::<Result<_, _>>()?;
        let expected: Vec<&str> = self.written.iter().map(|file| file.text.as_str()).collect();
        let held = ran.status.code() == Some(self.code)
            && out.get(..self.out.len()) == Some(self.out.as_str())
            && err.get(..self.err.len()) == Some(self.err)
            && written == expected;
        Ok((!held).then(|| format!("{:?}: exit {:?}\n{out}{err}{written:?}", self.arguments, ran.status.code())))
    }
}

impl File {
    fn at(path: &str, text: &str) -> File {
        File { path: path.to_owned(), text: text.to_owned() }
    }

    fn all<const COUNT: usize>(files: [[&str; 2]; COUNT]) -> Vec<File> {
        files.into_iter().map(|[path, text]| File::at(path, text)).collect()
    }
}
