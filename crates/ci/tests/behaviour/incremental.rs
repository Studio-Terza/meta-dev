use std::error::Error;
use std::ffi::OsString;
use std::process::ExitCode;

use ci::ports::Invocation;

use super::{ExactReply, Scratch, Script, Text};

const SURVIVOR: &str = r#"{"name": "crates/a/src/lib.rs:3:5: replace f -> bool with true", "file": "crates/a/src/lib.rs", "function": {"function_name": "f"}, "span": {"start": {"line": 3, "column": 5}}, "replacement": "true", "genre": "FnValue"}"#;
const SURVIVOR_MOVED: &str = r#"{"name": "crates/a/src/lib.rs:4:5: replace f -> bool with true", "file": "crates/a/src/lib.rs", "function": {"function_name": "f"}, "span": {"start": {"line": 4, "column": 5}}, "replacement": "true", "genre": "FnValue"}"#;
const ADDED: &str = r#"{"name": "crates/a/src/lib.rs:9:5: replace g -> u8 with 0", "file": "crates/a/src/lib.rs", "function": {"function_name": "g"}, "span": {"start": {"line": 9, "column": 5}}, "replacement": "0", "genre": "FnValue"}"#;
const ELSEWHERE: &str = r#"{"name": "crates/b/src/lib.rs:2:5: replace e -> u8 with 0", "file": "crates/b/src/lib.rs", "function": {"function_name": "e"}, "span": {"start": {"line": 2, "column": 5}}, "replacement": "0", "genre": "FnValue"}"#;
const UNTOUCHED: &str = r#"{"name": "crates/a/src/lib.rs:20:5: replace h -> u8 with 1", "file": "crates/a/src/lib.rs", "function": {"function_name": "h"}, "span": {"start": {"line": 20, "column": 5}}, "replacement": "1", "genre": "FnValue"}"#;
const CAUGHT_BEFORE: &str = r#"{"name": "crates/a/src/lib.rs:12:5: replace k -> u8 with 1", "file": "crates/a/src/lib.rs", "function": {"function_name": "k"}, "span": {"start": {"line": 12, "column": 5}}, "replacement": "1", "genre": "FnValue"}"#;

const TEST_ADDED: &str = "diff --git a/crates/a/tests/behaviour.rs b/crates/a/tests/behaviour.rs\n--- a/crates/a/tests/behaviour.rs\n+++ b/crates/a/tests/behaviour.rs\n@@ -9,0 +10,3 @@\n+#[test]\n+fn f_is_false() {\n+}\n";
const TEST_EDITED: &str = "diff --git a/crates/a/tests/behaviour.rs b/crates/a/tests/behaviour.rs\n--- a/crates/a/tests/behaviour.rs\n+++ b/crates/a/tests/behaviour.rs\n@@ -4 +4 @@\n-    assert!(f());\n+    assert!(g());\n";
const FAKE_EDITED: &str = "diff --git a/crates/fakes/src/lib.rs b/crates/fakes/src/lib.rs\n--- a/crates/fakes/src/lib.rs\n+++ b/crates/fakes/src/lib.rs\n@@ -2 +1,0 @@\n-pub fn faked() {}\n";
const SHARED_EDITED: &str = "diff --git a/crates/shared/src/lib.rs b/crates/shared/src/lib.rs\n--- a/crates/shared/src/lib.rs\n+++ b/crates/shared/src/lib.rs\n@@ -2 +1,0 @@\n-pub fn shared() {}\n";
const CODE_EDITED: &str = "diff --git a/crates/a/src/lib.rs b/crates/a/src/lib.rs\n--- a/crates/a/src/lib.rs\n+++ b/crates/a/src/lib.rs\n@@ -8 +8,2 @@\n-    old();\n+    new();\n+    newer();\n";
const NOTHING_MOVED: &str = "";
const LINE_ABOVE: &str = "diff --git a/crates/a/src/lib.rs b/crates/a/src/lib.rs\n--- a/crates/a/src/lib.rs\n+++ b/crates/a/src/lib.rs\n@@ -1,0 +2 @@\n+use std::fmt;\n";

const METADATA: &str = r#"{"workspace_root": "/w", "workspace_members": ["a 0.1.0 (path+file:///w/crates/a)", "fakes 0.1.0 (path+file:///w/crates/fakes)", "shared 0.1.0 (path+file:///w/crates/shared)"], "packages": [{"name": "a", "id": "a 0.1.0 (path+file:///w/crates/a)", "manifest_path": "/w/crates/a/Cargo.toml", "dependencies": [{"name": "fakes", "kind": "dev"}, {"name": "shared", "kind": null}, {"name": "shared", "kind": "dev"}, {"name": "serde", "kind": null}]}, {"name": "fakes", "id": "fakes 0.1.0 (path+file:///w/crates/fakes)", "manifest_path": "/w/crates/fakes/Cargo.toml", "dependencies": []}, {"name": "shared", "id": "shared 0.1.0 (path+file:///w/crates/shared)", "manifest_path": "/w/crates/shared/Cargo.toml", "dependencies": []}], "resolve": null}"#;

struct Prior {
    survivors: &'static [&'static str],
    timed_out: &'static [&'static str],
    since_head: &'static str,
    moved: &'static str,
    listed: Vec<&'static str>,
    since: Vec<&'static str>,
    earlier_tests: &'static str,
    since_tests: &'static str,
}

struct Planned {
    out: Vec<String>,
    selected: Vec<String>,
    shards: Vec<String>,
    wholes: usize,
}

impl Prior {
    const EMPTY_TREE: &'static str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

    fn outcomes(survivors: &[&str], timed_out: &[&str]) -> String {
        let summarized = |summary: &'static str| {
            move |mutant: &&str| ["{\"scenario\": {\"Mutant\": ", mutant, "}, \"summary\": \"", summary, "\"}"].concat()
        };
        let missed = survivors
            .iter()
            .map(summarized("MissedMutant"))
            .chain(timed_out.iter().map(summarized("Timeout")))
            .chain([["{\"scenario\": {\"Mutant\": ", CAUGHT_BEFORE, "}, \"phase_results\": [{\"phase\": \"Build\", \"duration\": 3000.0}, {\"phase\": \"Test\", \"duration\": 3000.0}], \"summary\": \"CaughtMutant\"}"].concat()]);
        [
            "{\"outcomes\": [{\"scenario\": \"Baseline\", \"summary\": \"Success\"}",
            &missed.map(|outcome| [", ", &outcome].concat()).collect::<String>(),
            "]}",
        ]
        .concat()
    }

    fn listing(mutants: &[&str]) -> String {
        mutants.first().map_or_else(String::new, |_| ["[", &mutants.join(", "), "]"].concat())
    }

    fn planned(&self) -> Result<Planned, Box<dyn Error>> {
        let scratch = Scratch::new()?;
        scratch.directory("prior/download/0")?;
        scratch.write("prior/tested", "t0\nh0\n")?;
        scratch.write("prior/download/0/outcomes.json", &Prior::outcomes(self.survivors, self.timed_out))?;
        scratch.write("metadata.json", METADATA)?;
        let since_diff = scratch.file("prior/since.diff");
        let diff = scratch.file("c.diff");
        let script = Script {
            changed: "+changed\n".to_owned(),
            exact: vec![
                ExactReply { asked: Invocation::new("git", ["rev-parse", "HEAD^2"]), text: "h1\n".to_owned() },
                ExactReply {
                    asked: Invocation::new(
                        "git",
                        ["log", "--first-parent", "--no-merges", "--format=", "-p", "-U0", "--no-renames", "h0..h1"],
                    ),
                    text: self.since_head.to_owned(),
                },
                ExactReply {
                    asked: Invocation::new("git", ["diff", "-U0", "--no-renames", "t0", "HEAD", "--", "*.rs"]),
                    text: self.moved.to_owned(),
                },
                ExactReply {
                    asked: Invocation::new("cargo", ["mutants", "--list", "--json", "--in-diff", diff.as_str()]),
                    text: Prior::listing(&self.listed),
                },
                ExactReply {
                    asked: Invocation::new("cargo", ["mutants", "--list", "--json", "--in-diff", since_diff.as_str()]),
                    text: Prior::listing(&self.since),
                },
                ExactReply {
                    asked: Invocation::new(
                        "git",
                        ["diff", "--name-only", "--no-renames", "t0^1", "t0", "--", ":(glob)*/**/tests/**/*.rs"],
                    ),
                    text: self.earlier_tests.to_owned(),
                },
                ExactReply {
                    asked: Invocation::new(
                        "git",
                        ["diff", "--name-only", "--no-renames", "t0", "HEAD", "--", ":(glob)*/**/tests/**/*.rs"],
                    ),
                    text: self.since_tests.to_owned(),
                },
            ],
            ..Script::new()
        };
        let prior = scratch.file("prior");
        let metadata = scratch.file("metadata.json");
        let answer = scratch.answer(
            &script,
            &["mutants-plan", &diff, &scratch.file("plan.txt"), "changed", "--prior", &prior, &metadata, "-p"],
        );
        assert_eq!(answer.exit, ExitCode::SUCCESS);
        Ok(Planned {
            out: Text::lines(&answer.out),
            selected: Text::lines(&scratch.read("prior/selected.txt")),
            shards: Text::lines(&scratch.read("plan.txt")),
            wholes: script
                .calls
                .borrow()
                .iter()
                .filter(|call| call.arguments().iter().any(|argument| argument == Prior::EMPTY_TREE))
                .count(),
        })
    }
}

#[test]
fn a_survivor_a_test_only_commit_fixes_is_selected_again_and_nothing_else_is() -> Result<(), Box<dyn Error>> {
    let planned = Prior {
        timed_out: &[],
        survivors: &[SURVIVOR],
        since_head: TEST_ADDED,
        moved: NOTHING_MOVED,
        listed: vec![SURVIVOR, CAUGHT_BEFORE, UNTOUCHED],
        since: Vec::new(),
        earlier_tests: "",
        since_tests: "",
    }
    .planned()?;
    assert_eq!(planned.selected, ["crates/a/src/lib.rs:3:5: replace f -> bool with true"]);
    assert_eq!(
        planned.out,
        ["mutants: 1 selected since h0: its 1 survivors and the lines changed since", "mutants: 1 to test in 1 shards"]
    );
    assert_eq!(planned.shards, [r#"shards=[{"index":0,"minutes":45}]"#]);
    Ok(())
}

#[test]
fn a_fix_commit_s_changed_lines_are_mutated_and_a_survivor_follows_its_line_down() -> Result<(), Box<dyn Error>> {
    let planned = Prior {
        timed_out: &[],
        survivors: &[SURVIVOR],
        since_head: CODE_EDITED,
        moved: LINE_ABOVE,
        listed: vec![SURVIVOR_MOVED, ADDED, CAUGHT_BEFORE, UNTOUCHED],
        since: vec![ADDED],
        earlier_tests: "",
        since_tests: "",
    }
    .planned()?;
    assert_eq!(
        planned.selected,
        ["crates/a/src/lib.rs:4:5: replace f -> bool with true", "crates/a/src/lib.rs:9:5: replace g -> u8 with 0"]
    );
    assert_eq!(planned.out.last().map(String::as_str), Some("mutants: 2 to test in 1 shards"));
    Ok(())
}

#[test]
fn a_prior_whose_survivors_are_all_caught_and_no_line_changed_selects_nothing() -> Result<(), Box<dyn Error>> {
    let planned = Prior {
        timed_out: &[],
        survivors: &[],
        since_head: TEST_ADDED,
        moved: NOTHING_MOVED,
        listed: vec![CAUGHT_BEFORE, UNTOUCHED],
        since: Vec::new(),
        earlier_tests: "",
        since_tests: "",
    }
    .planned()?;
    assert_eq!(planned.selected, Vec::<String>::new());
    assert_eq!(planned.shards, [r#"shards=[{"index":0,"minutes":45}]"#]);
    Ok(())
}

laws::table!(
    only_test_code_edited_other_than_by_additions_since_the_prior_falls_back_to_every_changed_line,
    |since: &&'static str| Prior { timed_out: &[], survivors: &[SURVIVOR], since_head: since, moved: NOTHING_MOVED, listed: vec![SURVIVOR, UNTOUCHED], since: Vec::new(), earlier_tests: "", since_tests: "" }
        .planned()
        .map_or_default(|planned| (planned.out.first().cloned().unwrap_or_default(), planned.selected.len())),
    [
        TEST_EDITED => ("mutants: every changed line, since test code changed other than by additions: crates/a/tests/behaviour.rs".to_owned(), 0),
        FAKE_EDITED => ("mutants: every changed line, since test code changed other than by additions: crates/fakes/src/lib.rs".to_owned(), 0),
        SHARED_EDITED => ("mutants: 1 selected since h0: its 1 survivors and the lines changed since".to_owned(), 1),
        "not a patch\n" => ("mutants: every changed line, since git printed a patch this cannot read".to_owned(), 0),
    ]
);

#[test]
fn without_a_kept_prior_the_plan_lists_every_changed_line_as_before() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.directory("prior")?;
    let diff = scratch.file("c.diff");
    let listed = Invocation::new("cargo", ["mutants", "--list", "--json", "--in-diff", diff.as_str()]);
    let script = Script {
        changed: "+changed\n".to_owned(),
        exact: vec![ExactReply { asked: listed, text: super::Listing::of(3) }],
        ..Script::new()
    };
    let prior = scratch.file("prior");
    let answer = scratch.answer(
        &script,
        &["mutants-plan", &diff, &scratch.file("plan.txt"), "changed", "--prior", &prior, "metadata.json", "-p"],
    );
    assert_eq!(
        Text::lines(&answer.out),
        ["mutants: every changed line, no earlier run's result to start from", "mutants: 3 to test in 1 shards"]
    );
    assert_eq!(scratch.kind_of("prior/selected.txt"), file_ports::Kind::Missing);
    Ok(())
}

#[derive(Clone, Copy, Debug)]
enum Ancestry {
    Ancestor,
    Diverged,
}

struct Earlier {
    ancestry: Ancestry,
    uploaded: &'static str,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Looked {
    out: Vec<String>,
    marked: Vec<String>,
    downloads: usize,
}

impl Ancestry {
    fn refused(self) -> Vec<Invocation> {
        match self {
            Ancestry::Ancestor => Vec::new(),
            Ancestry::Diverged => vec![Invocation::new("git", ["merge-base", "--is-ancestor", "h0", "h1"])],
        }
    }
}

impl Earlier {
    const RUNS: &'static str = r#"[{"databaseId": 7, "headSha": "h0"}]"#;
    const UPLOADED: &'static str = "[.artifacts[] | select(.expired | not) | .name | capture(\"^mutants-result-(?<index>[0-9]+)-of-(?<count>[0-9]+)$\") | {index: (.index | tonumber), count: (.count | tonumber)}]";

    fn looked(&self) -> Result<Looked, Box<dyn Error>> {
        let scratch = Scratch::new()?;
        scratch.directory("prior/download/0")?;
        scratch.directory("prior/download/1")?;
        scratch.write("prior/download/0/tested", "t0\nh0\n")?;
        scratch.write("prior/download/1/tested", "t0\nh0\n")?;
        let script = Script {
            exact: vec![
                ExactReply { asked: Invocation::new("git", ["rev-parse", "HEAD^2"]), text: "h1\n".to_owned() },
                ExactReply {
                    asked: Invocation::new(
                        "gh",
                        [
                            "run",
                            "list",
                            "--workflow",
                            "ci.yml",
                            "--event",
                            "pull_request",
                            "--status",
                            "completed",
                            "--branch",
                            "topic",
                            "--repo",
                            "o/r",
                            "--limit",
                            "10",
                            "--json",
                            "databaseId,headSha",
                        ],
                    ),
                    text: Earlier::RUNS.to_owned(),
                },
                ExactReply {
                    asked: Invocation::new(
                        "gh",
                        ["api", "repos/o/r/actions/runs/7/artifacts?per_page=100", "--jq", Earlier::UPLOADED],
                    ),
                    text: self.uploaded.to_owned(),
                },
            ],
            refused: self.ancestry.refused(),
            ..Script::new()
        };
        let answer = scratch.answer(&script, &["mutants-prior", "o/r", "topic", &scratch.file("prior")]);
        assert_eq!(answer.exit, ExitCode::SUCCESS);
        let downloads = script
            .calls
            .borrow()
            .iter()
            .filter(|call| call.arguments().get(..2) == Some(&[OsString::from("run"), OsString::from("download")][..]))
            .count();
        Ok(Looked { out: Text::lines(&answer.out), marked: Text::lines(&scratch.read("prior/tested")), downloads })
    }
}

laws::table!(
    only_a_complete_earlier_run_on_an_ancestor_of_the_head_is_reused,
    |earlier: &Earlier| earlier.looked().unwrap_or_default(),
    [
        Earlier { ancestry: Ancestry::Ancestor, uploaded: r#"[{"index": 0, "count": 2}, {"index": 1, "count": 2}]"# } => Looked { out: vec!["mutants: reusing run 7, which tested h0".to_owned()], marked: vec!["t0".to_owned(), "h0".to_owned()], downloads: 2 },
        Earlier { ancestry: Ancestry::Diverged, uploaded: r#"[{"index": 0, "count": 2}, {"index": 1, "count": 2}]"# } => Looked { out: vec!["mutants: no earlier run on an ancestor of this head kept every shard's result, so this run mutates every changed line".to_owned()], marked: Vec::new(), downloads: 0 },
        Earlier { ancestry: Ancestry::Ancestor, uploaded: r#"[{"index": 1, "count": 2}]"# } => Looked { out: vec!["mutants: no earlier run on an ancestor of this head kept every shard's result, so this run mutates every changed line".to_owned()], marked: Vec::new(), downloads: 0 },
        Earlier { ancestry: Ancestry::Ancestor, uploaded: "[]" } => Looked { out: vec!["mutants: no earlier run on an ancestor of this head kept every shard's result, so this run mutates every changed line".to_owned()], marked: Vec::new(), downloads: 0 },
    ]
);

#[test]
fn a_shard_mutates_only_the_selected_names_and_keeps_what_it_left() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.directory("prior")?;
    scratch.write("prior/selected.txt", "crates/a/src/lib.rs:3:5: replace f -> bool with true")?;
    let script = Script {
        exact: vec![ExactReply {
            asked: Invocation::new("git", ["rev-parse", "HEAD", "HEAD^2"]),
            text: "t1\nh1\n".to_owned(),
        }],
        ..Script::new()
    };
    let arguments = [
        "mutants-changed",
        &scratch.file("c.diff"),
        &scratch.file("out"),
        "0/1",
        "--selected",
        &scratch.file("prior/selected.txt"),
        "--kept",
        &scratch.file("kept"),
        "-p",
    ];
    let answer = scratch.answer(&script, &arguments);
    assert_eq!(answer.exit, ExitCode::SUCCESS);
    let mutated: Vec<String> = script.calls.borrow().first().map_or_default(|call| {
        call.arguments().iter().map(|argument| argument.to_string_lossy().into_owned()).collect()
    });
    assert_eq!(
        mutated.get(..5).map_or_default(<[String]>::to_vec),
        [
            "mutants",
            "--in-diff",
            &scratch.file("c.diff"),
            "--re",
            "^crates/a/src/lib\\.rs:3:5: replace f \\-> bool with true$"
        ]
    );
    assert_eq!(
        (Text::lines(&scratch.read("kept/tested")), scratch.kind_of("kept/outcomes.json")),
        (vec!["t1".to_owned(), "h1".to_owned()], file_ports::Kind::Missing)
    );
    Ok(())
}

#[test]
fn a_shard_given_an_empty_selection_mutates_nothing_and_keeps_an_empty_result() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.directory("prior")?;
    scratch.write("prior/selected.txt", "")?;
    let script = Script {
        exact: vec![ExactReply {
            asked: Invocation::new("git", ["rev-parse", "HEAD", "HEAD^2"]),
            text: "t1\nh1\n".to_owned(),
        }],
        ..Script::new()
    };
    let arguments = [
        "mutants-changed",
        &scratch.file("c.diff"),
        &scratch.file("out"),
        "0/1",
        "--selected",
        &scratch.file("prior/selected.txt"),
        "--kept",
        &scratch.file("kept"),
        "-p",
    ];
    let answer = scratch.answer(&script, &arguments);
    assert_eq!(
        (answer.exit, Text::lines(&answer.out)),
        (
            ExitCode::SUCCESS,
            vec!["mutants: none selected, no changed line since the prior run and no survivor of it".to_owned()]
        )
    );
    let programs: Vec<String> =
        script.calls.borrow().iter().map(|call| call.program().to_string_lossy().into_owned()).collect();
    assert_eq!(programs, ["git"]);
    assert_eq!(
        (Text::lines(&scratch.read("kept/tested")), scratch.kind_of("kept/outcomes.json")),
        (vec!["t1".to_owned(), "h1".to_owned()], file_ports::Kind::Missing)
    );
    Ok(())
}

#[test]
fn a_survivor_the_scoped_pass_still_misses_is_kept_for_the_next_push() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.directory("out/mutants.out")?;
    scratch.directory("out/scoped/mutants.out")?;
    scratch.write("out/mutants.out/missed.txt", "crates/a/src/lib.rs:3:5: replace f -> bool with true\n")?;
    scratch.write("out/scoped/mutants.out/missed.txt", "crates/a/src/lib.rs:3:5: replace f -> bool with true\n")?;
    let outcomes = Prior::outcomes(&[SURVIVOR], &[]);
    scratch.write("out/scoped/mutants.out/outcomes.json", &outcomes)?;
    let script = Script {
        exact: vec![ExactReply {
            asked: Invocation::new("git", ["rev-parse", "HEAD", "HEAD^2"]),
            text: "t1\nh1\n".to_owned(),
        }],
        ..Script::exiting(2)
    };
    let arguments = [
        "mutants-changed",
        &scratch.file("c.diff"),
        &scratch.file("out"),
        "0/1",
        "--kept",
        &scratch.file("kept"),
        "-p",
    ];
    let answer = scratch.answer(&script, &arguments);
    assert_eq!(answer.exit, ExitCode::from(2));
    assert_eq!(
        (Text::lines(&scratch.read("kept/tested")), scratch.read("kept/outcomes.json")),
        (vec!["t1".to_owned(), "h1".to_owned()], outcomes)
    );
    Ok(())
}

struct Retested {
    earlier: &'static str,
    since: &'static str,
}

impl Retested {
    fn wholes(&self) -> usize {
        Prior {
            timed_out: &[],
            survivors: &[],
            since_head: TEST_ADDED,
            moved: NOTHING_MOVED,
            listed: vec![UNTOUCHED],
            since: Vec::new(),
            earlier_tests: self.earlier,
            since_tests: self.since,
        }
        .planned()
        .map_or_default(|planned| planned.wholes)
    }
}

laws::table!(
    a_crate_s_whole_source_is_mutated_only_when_its_tests_first_change_after_the_earlier_run,
    Retested::wholes,
    [
        Retested { earlier: "", since: "crates/a/tests/behaviour.rs\n" } => 1,
        Retested { earlier: "crates/a/tests/behaviour.rs\n", since: "crates/a/tests/behaviour.rs\n" } => 0,
        Retested { earlier: "crates/a/tests/behaviour.rs\n", since: "" } => 0,
    ]
);

#[test]
fn a_survivor_that_timed_out_in_the_scope_is_selected_again() -> Result<(), Box<dyn Error>> {
    let planned = Prior {
        survivors: &[],
        timed_out: &[SURVIVOR],
        since_head: TEST_ADDED,
        moved: NOTHING_MOVED,
        listed: vec![SURVIVOR, CAUGHT_BEFORE, UNTOUCHED],
        since: Vec::new(),
        earlier_tests: "",
        since_tests: "",
    }
    .planned()?;
    assert_eq!(planned.selected, ["crates/a/src/lib.rs:3:5: replace f -> bool with true"]);
    Ok(())
}

#[test]
fn a_full_plan_sizes_each_shard_from_the_prior_s_measured_cost_per_crate() -> Result<(), Box<dyn Error>> {
    let planned = Prior {
        timed_out: &[],
        survivors: &[SURVIVOR],
        since_head: TEST_EDITED,
        moved: NOTHING_MOVED,
        listed: vec![SURVIVOR, UNTOUCHED, ELSEWHERE],
        since: Vec::new(),
        earlier_tests: "",
        since_tests: "",
    }
    .planned()?;
    assert_eq!(planned.shards, [r#"shards=[{"index":0,"minutes":303}]"#]);
    Ok(())
}
