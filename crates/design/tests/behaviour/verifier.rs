use std::ffi::OsString;
use std::path::Path;

use console_ports::Variables;
use design::{Agents, Count, Failure, Outcome};
use design_model::{FeatureName, Verdict};
use design_store::{Evidence, IdealId, Judgment, Judgments, LineNumber, Revision, Ruling, SourcePath, StreamSchema};
use dispatch_fakes::{Scripted, Stepped};
use dispatch_ports::{Finished, Model, Output, Pause, Prompt, Reply, Tokens, Usage, Wall};
use file_ports::{Bytes, Files, Memory};
use newtype::Wrapping;
use port_core::Run;
use store_ports::Store;

use crate::{Opening, Sqlite};

use super::{Answered, FakeLent, HOME, Invocation, Refusal, Session, Speaker};

struct Verifying {
    store: Sqlite,
    files: Memory,
    script: Scripted,
}

struct Verified {
    outcome: Result<Outcome, Failure>,
    text: String,
}

impl Verifying {
    const OBJECT: &str = "0123456789abcdef0123456789abcdef01234567";
    const PACKED: &str = "fedcba9876543210fedcba9876543210fedcba98";
    const SAID: [(&str, &str, &str); 5] = [
        ("o1", "every check runs in CI and nothing gets past it", "2026-01-01"),
        ("o2", "models quote the owner and never write their own prose", "2026-01-02"),
        ("o3", "checks may now be skipped when CI is slow", "2026-01-03"),
        ("o4", "tests go through the real CLI", "2026-01-04"),
        ("o5", "no exceptions to any check", "2026-01-05"),
    ];
    const DRAFTED: &str = r#"[
        {"quotes": [{"uuid": "o1", "quote": "every check runs in CI"}]},
        {"quotes": [{"uuid": "o2", "quote": "models quote the owner"}, {"uuid": "o2", "quote": "models paraphrase freely"}]},
        {"quotes": [{"uuid": "o3", "quote": "checks may now be skipped when CI is slow"}]},
        {"quotes": [{"uuid": "o1", "quote": "an invented rule"}]},
        {"quotes": [{"uuid": "o4", "quote": "tests go through the real CLI"}]},
        {"quotes": [{"uuid": "o5", "quote": "no exceptions to any check"}]}
    ]"#;
    const GROUPED: &str = r#"[{"ideals": [3], "supersedes": [1]}, {"ideals": [2], "supersedes": [3]}]"#;
    const PROPOSED: &str = r#"[
        {"name": "ideal-verifier", "consideration": "checks",
         "quotes": [{"uuid": "o2", "quote": "models quote the owner"}, {"uuid": "o2", "quote": "an excerpt never said"}],
         "crates": ["design", "no-such-crate"]},
        {"name": "ghost", "consideration": "process", "quotes": [{"uuid": "o1", "quote": "nothing like this"}], "crates": ["design"]},
        {"name": "cooking", "consideration": "kitchen", "quotes": [{"uuid": "o1", "quote": "every check runs in CI"}]}
    ]"#;
    const RULED: &str = r#"[
        {"ideal": 1, "verdict": "holds", "evidence": []},
        {"ideal": 2, "verdict": "violated", "evidence": [
            {"file": "crates/design/src/lib.rs", "line": 2},
            {"file": "crates/design/src/lib.rs", "line": 40},
            {"file": "crates/design/src/gone.rs", "line": 1}]},
        {"ideal": 3, "verdict": "holds", "evidence": [{"file": "crates/design/src/cli.rs", "line": 1}]},
        {"ideal": 4, "verdict": "violated", "evidence": [{"file": "crates/design/src/lib.rs", "line": 99}]}
    ]"#;

    fn over(replies: &[&str]) -> Result<Verifying, Failure> {
        let lines: String = Verifying::SAID
            .iter()
            .map(|(uuid, text, day)| Session::stamped(Speaker::Owner, uuid, "null", text, day))
            .collect();
        let outputs = replies.iter().map(|reply| Some(Output::Finished(Verifying::finished(reply))));
        let verifying = Verifying {
            store: Sqlite::in_memory::<StreamSchema>(),
            files: Session::working(lines).files(),
            script: Scripted::replying(outputs.collect::<Vec<Option<Output>>>()),
        };
        Invocation::within(&verifying.store, FakeLent::new(&verifying.files), &["ingest"])?;
        Ok(verifying)
    }

    fn finished(reply: &str) -> Finished {
        Finished::new(
            Reply::new(reply.to_owned()),
            Usage::new(Tokens::new(10), Tokens::new(20), Tokens::new(3), Tokens::new(4)),
        )
    }

    fn placed(&self, path: &str, text: &str) -> Result<(), laws::Unmet> {
        let placed = Path::new(path);
        self.files.create_directories(placed.parent().unwrap_or(placed))?;
        self.files.write(placed, Bytes::from(text))?;
        Ok(())
    }

    fn crated(&self) -> Result<(), laws::Unmet> {
        self.placed("/work/crates/design/Cargo.toml", "[package]\n")?;
        self.placed("/work/crates/other/Cargo.toml", "[package]\n")?;
        self.placed("/work/crates/unmanifested/README", "no manifest\n")?;
        self.placed("/work/crates/design/src/lib.rs", "mod cli;\nfn main() {}\nconst LIMIT: u8 = 1;\n")?;
        self.placed("/work/crates/design/src/cli.rs", "pub struct Cli;\n")?;
        self.placed("/work/crates/design/src/notes.txt", "not rust\n")
    }

    fn committed(&self) -> Result<(), laws::Unmet> {
        self.placed("/work/.git/HEAD", "ref: refs/heads/main\n")?;
        self.placed("/work/.git/refs/heads/main", &format!("{}\n", Verifying::OBJECT))
    }

    fn ran(&self, arguments: &[&str]) -> Verified {
        let environment = [("HOME", HOME), ("PWD", "/work")]
            .map(|(name, value)| (OsString::from(name), OsString::from(value)))
            .into_iter()
            .collect::<Variables>();
        let uttered = design::Cli::new(
            &environment,
            FakeLent::new(&self.files),
            Opening(|_path| self.store.clone()),
            Agents::new(|_scratch| &self.script, Stepped::starting(Wall::new(1_800_000_000), Pause::new(0))),
        )
        .report(&Run::root(), arguments.iter().map(OsString::from));
        Verified { text: Answered::said(&uttered), outcome: uttered.ended() }
    }

    fn prompts(&self) -> Vec<String> {
        self.script.prompts().into_iter().map(Prompt::into_inner).collect()
    }

    fn judged(&self) -> Result<Vec<Judgment>, laws::Unmet> {
        let run = Run::root();
        let judged = run.port(self.store.clone()).read(Judgments)?;
        run.finish().settled()?;
        Ok(judged)
    }
}

#[test]
fn haiku_drafts_ideals_in_the_owners_words_only_and_a_later_contradiction_supersedes_the_earlier()
-> Result<(), laws::Unmet> {
    let verifying = Verifying::over(&[Verifying::DRAFTED, Verifying::GROUPED])?;
    let ran = verifying.ran(&["ideals"]);
    assert_eq!(ran.outcome?, Outcome::Idealised { ideals: Count::new(5) });
    golden::golden!("ideals.txt", &ran.text);
    assert_eq!(verifying.script.models(), [Model::Haiku, Model::Haiku]);
    let prompts = verifying.prompts();
    let drafting = prompts.first().cloned().unwrap_or_default();
    let grouping = prompts.get(1).cloned().unwrap_or_default();
    assert!(drafting.contains("[o2 2026-01-02T00:00:00Z] models quote the owner and never write"), "{drafting}");
    assert!(grouping.contains("1.\n   o1 2026-01-01T00:00:00Z: \"every check runs in CI\"\n2.\n"), "{grouping}");
    assert!(grouping.contains("4.\n   o4 2026-01-04T00:00:00Z: \"tests go through the real CLI\"\n5."), "{grouping}");
    Ok(())
}

#[test]
fn ideals_run_again_replace_the_stored_ones() -> Result<(), laws::Unmet> {
    let verifying = Verifying::over(&[Verifying::DRAFTED, Verifying::GROUPED, Verifying::DRAFTED, "[]"])?;
    verifying.ran(&["ideals"]).outcome?;
    let again = verifying.ran(&["ideals"]);
    assert_eq!(again.outcome?, Outcome::Idealised { ideals: Count::new(5) });
    assert!(again.text.contains("i1\tcurrent\to1\t\"every check runs in CI\"\n"), "{}", again.text);
    Ok(())
}

#[test]
fn sonnet_proposes_features_whose_quotes_and_crates_are_checked_in_code() -> Result<(), laws::Unmet> {
    let verifying = Verifying::over(&[Verifying::PROPOSED])?;
    verifying.crated()?;
    let ran = verifying.ran(&["features"]);
    assert_eq!(ran.outcome?, Outcome::Featured { features: Count::new(1) });
    golden::golden!("features.txt", &ran.text);
    assert_eq!(verifying.script.models(), [Model::Sonnet]);
    let proposing = verifying.prompts().first().cloned().unwrap_or_default();
    assert!(proposing.contains("Crates: design, other\n"), "{proposing}");
    assert!(proposing.contains("[o5 2026-01-05T00:00:00Z] no exceptions to any check"), "{proposing}");
    Ok(())
}

#[test]
fn sonnet_judges_one_feature_against_the_current_ideals_and_only_real_lines_count_as_evidence()
-> Result<(), laws::Unmet> {
    let verifying = Verifying::over(&[Verifying::DRAFTED, Verifying::GROUPED, Verifying::PROPOSED, Verifying::RULED])?;
    verifying.crated()?;
    verifying.committed()?;
    verifying.ran(&["ideals"]).outcome?;
    verifying.ran(&["features"]).outcome?;
    let judged = verifying.ran(&["judge", "ideal-verifier"]);
    let listed = verifying.ran(&["judgments"]);
    assert_eq!((judged.outcome?, listed.outcome?), (Outcome::Judged { findings: Count::new(3) }, Outcome::Judgments));
    golden::golden!("judgments.txt", &judged.text);
    golden::golden!("judgments.txt", &listed.text);
    assert_eq!(verifying.script.models(), [Model::Haiku, Model::Haiku, Model::Sonnet, Model::Sonnet]);
    let judging = verifying.prompts().get(3).cloned().unwrap_or_default();
    assert!(judging.contains("== crates/design/src/cli.rs ==\n1: pub struct Cli;\n"), "{judging}");
    assert!(judging.contains("== crates/design/src/lib.rs ==\n1: mod cli;\n2: fn main() {}\n3: const"), "{judging}");
    assert!(judging.contains("i2:\n   o2 2026-01-02T00:00:00Z: \"models quote the owner\"\n"), "{judging}");
    assert_eq!((judging.contains("i1:"), judging.contains("notes.txt")), (false, false), "{judging}");
    Ok(())
}

#[test]
fn a_judgment_is_stored_with_the_commit_git_has_checked_out() -> Result<(), laws::Unmet> {
    let verifying = Verifying::over(&[Verifying::DRAFTED, Verifying::GROUPED, Verifying::PROPOSED, Verifying::RULED])?;
    verifying.crated()?;
    verifying.committed()?;
    verifying.ran(&["ideals"]).outcome?;
    verifying.ran(&["features"]).outcome?;
    verifying.ran(&["judge", "ideal-verifier"]).outcome?;
    let feature: FeatureName = "ideal-verifier".parse()?;
    let revision = Revision::try_new(Verifying::OBJECT.to_owned())?;
    let cited = Evidence::new(SourcePath::try_new("crates/design/src/lib.rs".to_owned())?, LineNumber::new(2));
    let first = Ruling::new(IdealId::new(2), Verdict::Violated, Some(cited)).judged(feature, revision);
    assert_eq!(verifying.judged()?.first(), Some(&first));
    Ok(())
}

#[test]
fn a_worktree_judgment_reads_its_commit_through_the_common_git_directory_and_packed_refs() -> Result<(), laws::Unmet> {
    let verifying = Verifying::over(&[Verifying::DRAFTED, Verifying::GROUPED, Verifying::PROPOSED, Verifying::RULED])?;
    verifying.crated()?;
    verifying.placed("/work/.git", "gitdir: /repo/.git/worktrees/wt\n")?;
    verifying.placed("/repo/.git/worktrees/wt/HEAD", "ref: refs/heads/design/verifier\n")?;
    verifying.placed("/repo/.git/worktrees/wt/commondir", "../..\n")?;
    verifying.placed(
        "/repo/.git/packed-refs",
        &format!("# pack-refs with: peeled\n{} refs/heads/design/verifier\n", Verifying::PACKED),
    )?;
    verifying.ran(&["ideals"]).outcome?;
    verifying.ran(&["features"]).outcome?;
    verifying.ran(&["judge", "ideal-verifier"]).outcome?;
    let feature: FeatureName = "ideal-verifier".parse()?;
    let revision = Revision::try_new(Verifying::PACKED.to_owned())?;
    let missing = Ruling::new(IdealId::new(5), Verdict::Unclear, None).judged(feature, revision);
    assert_eq!(verifying.judged()?.last(), Some(&missing));
    Ok(())
}

#[test]
fn judging_a_feature_the_store_does_not_hold_or_without_a_git_commit_is_refused() -> Result<(), laws::Unmet> {
    let verifying = Verifying::over(&[Verifying::DRAFTED, Verifying::GROUPED, Verifying::PROPOSED, Verifying::RULED])?;
    verifying.crated()?;
    verifying.ran(&["ideals"]).outcome?;
    verifying.ran(&["features"]).outcome?;
    let unknown = verifying.ran(&["judge", "no-such-feature"]);
    let uncommitted = verifying.ran(&["judge", "ideal-verifier"]);
    assert_eq!(
        [Refusal::said(&unknown.outcome), Refusal::said(&uncommitted.outcome)],
        [
            "no feature no-such-feature in the design store; run design features first",
            "cannot read the checked-out commit of \"/work\" from git; nothing was judged",
        ]
    );
    assert_eq!(verifying.script.models().len(), 3);
    Ok(())
}
