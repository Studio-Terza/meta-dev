use meta_cli::{Answer, Call, Code, answer};
use meta_core::{Attempt, Event, Outcome, PrNumber, Record, Route, RunCount, Sha, Timestamp};
use meta_ports::memory::{FixedClock, MemoryLog, RecordedGit, RecordedGitHub};
use meta_ports::real::JsonlLog;
use meta_ports::{GhReplies, PortError, Ports};
use std::str::FromStr;

const RULES: &str = include_str!("../../../self/rules.toml");
const ORIGIN: &str = "https://github.com/Studio-Terza/meta-dev.git";
const REPO: &str = "Studio-Terza/meta-dev";
const PINNED: &str = "aaaaaaa";
const MOVED: &str = "bbbbbbb";
const REQUIRED: &str = "gates";
const ALLOW: &str = "{}\n";

type Memory = Ports<MemoryLog, RecordedGit, RecordedGitHub, FixedClock>;

fn id<T: FromStr>(text: &str) -> T
where
    T::Err: std::fmt::Debug,
{
    text.parse().unwrap()
}

fn opened(task: &str) -> Event {
    Event::Opened { task: id(task), title: id("Fix the hook") }
}

fn claimed(task: &str) -> Event {
    Event::Claimed { task: id(task), agent: id("alice") }
}

fn leased(lease: &str, task: &str) -> Event {
    Event::Leased { lease: id(lease), task: id(task) }
}

fn linked(task: &str, pr: &str) -> Event {
    Event::Linked { task: id(task), pr: id(pr), pin: id(PINNED) }
}

fn merged(pr: &str, runs: usize) -> Event {
    Event::Merged { pr: id(pr), head: id(PINNED), runs: RunCount::of(runs), route: Route::Meta }
}

fn merged_outside(pr: &str, runs: usize) -> Event {
    Event::Merged { pr: id(pr), head: id(PINNED), runs: RunCount::of(runs), route: Route::Outside }
}

fn done(task: &str) -> Event {
    Event::Done { task: id(task) }
}

fn escaped(reason: &str) -> Event {
    Event::Escaped { reason: id(reason) }
}

fn claimed_task() -> Vec<Event> {
    vec![opened("task-1"), claimed("task-1"), leased("lease-1", "task-1")]
}

fn linked_task() -> Vec<Event> {
    [claimed_task(), vec![linked("task-1", "7")]].concat()
}

fn rollup(outcome: &Outcome) -> serde_json::Value {
    let (status, conclusion) = match outcome {
        Outcome::Green => ("COMPLETED", "SUCCESS"),
        Outcome::Red => ("COMPLETED", "FAILURE"),
        Outcome::Pending => ("IN_PROGRESS", ""),
    };
    serde_json::json!({ "__typename": "CheckRun", "name": REQUIRED, "status": status, "conclusion": conclusion })
}

struct Gh<'a> {
    head: &'a str,
    branch: &'a str,
    base: &'a str,
    state: &'a str,
    rollup: Vec<serde_json::Value>,
    required: Result<Vec<&'a str>, &'a str>,
}

impl Gh<'_> {
    fn replies(self) -> GhReplies {
        let view = serde_json::json!({
            "headRefOid": self.head,
            "headRefName": self.branch,
            "baseRefName": self.base,
            "state": self.state,
            "url": format!("https://github.com/{REPO}/pull/7"),
            "statusCheckRollup": self.rollup,
        });
        let required = self
            .required
            .map(|contexts| serde_json::json!({ "strict": true, "contexts": contexts, "checks": [] }).to_string())
            .map_err(PortError::new);
        let runs = serde_json::json!([{ "databaseId": 1 }, { "databaseId": 2 }, { "databaseId": 3 }]).to_string();
        GhReplies { view: view.to_string(), required, runs }
    }
}

fn gh<'a>(head: &'a str, outcomes: &[Outcome]) -> Gh<'a> {
    Gh {
        head,
        branch: "task-1",
        base: "main",
        state: "OPEN",
        rollup: outcomes.iter().map(rollup).collect(),
        required: Ok(vec![REQUIRED]),
    }
}

fn pull(head: &str, outcomes: &[Outcome]) -> GhReplies {
    gh(head, outcomes).replies()
}

fn given(events: Vec<Event>, pulls: Vec<(&str, GhReplies)>) -> Memory {
    let at = Timestamp::from_unix_seconds(1);
    Ports {
        log: MemoryLog { records: events.into_iter().map(|event| Record { at, event }).collect(), broken: None },
        git: RecordedGit { origin: id(ORIGIN), rules: RULES.to_owned() },
        github: RecordedGitHub {
            replies: pulls.into_iter().map(|(pr, replies)| (id::<PrNumber>(pr), replies)).collect(),
            merged: Vec::new(),
            listed: Some(Ok("[]".to_owned())),
        },
        clock: FixedClock(Timestamp::from_unix_seconds(2)),
    }
}

fn call(ports: &mut Memory, args: &[&str], stdin: &str) -> (Answer, Vec<Event>) {
    let before = ports.log.records.len();
    let words: Vec<String> = args.iter().map(|word| (*word).to_owned()).collect();
    let reply = answer(Call::parse(&words, || stdin.to_owned()).unwrap(), ports);
    let added = ports.log.records[before..].iter().map(|record| record.event.clone()).collect();
    (reply, added)
}

fn hook(events: Vec<Event>, input: &str) -> (Answer, Vec<Event>) {
    call(&mut given(events, Vec::new()), &["hook", "pre-tool-use"], input)
}

fn agent(prompt: &str) -> String {
    serde_json::json!({ "tool_name": "Agent", "tool_input": { "prompt": prompt } }).to_string()
}

fn deny(reason: &str) -> String {
    format!(
        r#"{{"hookSpecificOutput":{{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"{reason}"}}}}"#
    ) + "\n"
}

fn no_lease(why: &str) -> String {
    deny(&format!("{why} Get one with meta claim <task> --agent <name>, then put its lease id in the Agent prompt."))
}

fn refused(text: &str) -> Answer {
    Answer { text: format!("refused: {text}\n"), code: Code::Refused }
}

fn said(text: &str) -> Answer {
    Answer { text: format!("{text}\n"), code: Code::Done }
}

fn brief(agent: &str, lease: &str) -> String {
    format!(
        "task: task-1: Fix the hook
agent: {agent}
lease: {lease} (put it in the Agent prompt; it works once)
setup:
  1. git clone {ORIGIN} task-1
  2. cd task-1 && git switch -c task-1 origin/main
  3. set the git identity that rule identity names
  4. push, open the PR with gh pr create, then run meta link task-1 <pr>
rules:
  - identity: Commit as Cinglein <64114772+Cinglein@users.noreply.github.com>; never any other email; no personal or machine details in files.
    source: owner 2026-10-26: personal email leaked into public history
  - no-subagents: Do not spawn agents.
    source: owner 2026-10-28: CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH=1; one centrally allocated pool
  - pr-shape: One PR per coherent change, branched off main, however large. Do not stack.
    source: owner 2026-10-30: mega PRs are fine; CI is the only real gate; stacks churn CI
  - push-and-stop: Push, open the PR, run meta link, then stop. Do not wait on CI. Never merge.
    source: failure 2026-10-30: long-lived agents waiting on CI inflated context and waited on runs that never started
  - budgets: Never change a budget or gate.
    source: owner 2026-10-27: budgets and gates are owner-only
  - cargo-niceness: Run cargo under nice -n 19 with CARGO_BUILD_JOBS=2, one command at a time.
    source: owner 2026-10-28: thermal management
  - no-email: Send no email. Web requests use the User-Agent basho-trip-planner.
    source: owner 2026-10: privacy
"
    )
}

#[test]
fn open_prints_a_new_task_id_and_records_opened() {
    let (reply, events) = call(&mut given(vec![opened("task-1")], Vec::new()), &["open", "Fix", "the", "hook"], "");
    assert_eq!(reply, said("task-2"));
    assert_eq!(events, vec![opened("task-2")]);
}

#[test]
fn claim_of_an_open_unclaimed_task_leases_and_prints_the_brief_with_every_sourced_rule() {
    let (reply, events) =
        call(&mut given(vec![opened("task-1")], Vec::new()), &["claim", "task-1", "--agent", "alice"], "");
    assert_eq!(events, vec![claimed("task-1"), leased("lease-1", "task-1")]);
    assert_eq!(reply, Answer { text: brief("alice", "lease-1"), code: Code::Done });
}

#[test]
fn claim_of_a_claimed_done_or_unknown_task_is_refused_with_the_reason() {
    let finished = [linked_task(), vec![merged("7", 1), done("task-1")]].concat();
    let cases = [
        (
            claimed_task(),
            "task-1",
            "task-1 is already claimed and lease-1 is unused (task-1 claimed by alice: Fix the hook). Next: put lease-1 in the Agent prompt; once it is spent, meta claim task-1 --agent <name> issues a fresh lease.",
        ),
        (finished, "task-1", "task-1 is done. Next: meta open <title>."),
        (Vec::new(), "task-9", "task-9 is unknown. Next: meta status lists open tasks."),
    ];
    cases.into_iter().for_each(|(events, task, reason)| {
        let (reply, added) = call(&mut given(events, Vec::new()), &["claim", task, "--agent", "bob"], "");
        assert_eq!((reply, added), (refused(reason), Vec::new()));
    });
}

#[test]
fn claim_of_a_claimed_task_whose_lease_is_spent_issues_a_fresh_lease() {
    let spent = Event::LeaseUsed { lease: id("lease-1") };
    [claimed_task(), linked_task()].into_iter().for_each(|history| {
        let history = [history, vec![spent.clone()]].concat();
        let (reply, events) = call(&mut given(history, Vec::new()), &["claim", "task-1", "--agent", "bob"], "");
        assert_eq!(
            (reply.text, reply.code, events),
            (brief("bob", "lease-2"), Code::Done, vec![leased("lease-2", "task-1")])
        );
    });
}

#[test]
fn link_of_a_claimed_task_records_linked_with_the_head_pinned() {
    let mut ports = given(claimed_task(), vec![("7", pull(PINNED, &[]))]);
    let (reply, events) = call(&mut ports, &["link", "task-1", "7"], "");
    assert_eq!(reply, said("task-1 linked to #7, head pinned at aaaaaaa"));
    assert_eq!(events, vec![linked("task-1", "7")]);
}

#[test]
fn merge_of_a_linked_green_unmoved_pr_squash_merges_with_the_head_pinned() {
    let mut ports = given(linked_task(), vec![("7", pull(PINNED, &[Outcome::Green, Outcome::Green]))]);
    let (reply, events) = call(&mut ports, &["merge", "7"], "");
    assert_eq!(reply, said("#7 squash-merged at aaaaaaa; task-1 done"));
    assert_eq!(events, vec![merged("7", 3), done("task-1")]);
    assert_eq!(ports.github.merged, vec![(id::<PrNumber>("7"), id::<Sha>(PINNED))]);
}

#[test]
fn merge_of_an_unlinked_red_missing_or_moved_pr_is_refused_naming_which_with_the_next_step() {
    const RED: &str = "#7 cannot merge:\nred check gates on aaaaaaa. Next: fix it, push, then meta link task-1 7.";
    const WAIT: &str = "#7 cannot merge:\nmissing check gates on aaaaaaa. Next: wait for CI, then meta merge 7 again.";
    let no_checks = "#7 cannot merge:\nmissing check: no required checks are configured. Next: require checks on main in branch protection, then meta merge 7 again.";
    let moved = "head moved from pinned aaaaaaa to bbbbbbb. Next: review the new head, then meta link task-1 7.";
    let cases = [
        (
            "8",
            pull(PINNED, &[Outcome::Green]),
            "#8 is unlinked. Next: meta link <task> 8 from the agent that claimed the task.".to_owned(),
        ),
        ("7", pull(PINNED, &[Outcome::Red]), RED.to_owned()),
        ("7", pull(PINNED, &[]), WAIT.to_owned()),
        ("7", pull(PINNED, &[Outcome::Pending]), WAIT.to_owned()),
        ("7", pull(PINNED, &[Outcome::Green, Outcome::Red]), RED.to_owned()),
        ("7", pull(PINNED, &[Outcome::Red, Outcome::Green]), RED.to_owned()),
        ("7", pull(PINNED, &[Outcome::Pending, Outcome::Green]), WAIT.to_owned()),
        ("7", pull(PINNED, &[Outcome::Green, Outcome::Pending]), WAIT.to_owned()),
        ("7", pull(MOVED, &[Outcome::Green]), format!("#7 cannot merge:\n{moved}")),
        ("7", Gh { required: Ok(Vec::new()), ..gh(PINNED, &[Outcome::Green]) }.replies(), no_checks.to_owned()),
        (
            "7",
            Gh { state: "CLOSED", ..gh(PINNED, &[Outcome::Green]) }.replies(),
            "#7 is closed. Next: reopen it, then meta link task-1 7.".to_owned(),
        ),
        (
            "7",
            Gh { state: "MERGED", ..gh(MOVED, &[Outcome::Green]) }.replies(),
            format!("#7 was merged outside meta: {moved}"),
        ),
    ];
    cases.into_iter().for_each(|(pr, recorded, text)| {
        let mut ports = given(linked_task(), vec![(pr, recorded)]);
        let (reply, events) = call(&mut ports, &["merge", pr], "");
        assert_eq!((reply, events), (refused(&text), Vec::new()));
        assert!(ports.github.merged.is_empty());
    });
}

#[test]
fn escape_records_escaped_and_prints_open_count_and_repeats_of_the_reason() {
    let mut ports = given(vec![escaped("gh is down"), escaped("hook too strict")], Vec::new());
    let (reply, events) = call(&mut ports, &["escape", "gh", "is", "down"], "");
    assert_eq!(events, vec![escaped("gh is down")]);
    assert_eq!(reply, said("escape recorded: gh is down\nopen escapes: 3\nthis reason repeated: 1"));
}

#[test]
fn status_prints_open_tasks_live_leases_open_escapes_and_the_measures() {
    let history = [
        claimed_task(),
        vec![
            opened("task-2"),
            claimed("task-2"),
            leased("lease-2", "task-2"),
            Event::LeaseUsed { lease: id("lease-2") },
        ],
        vec![linked("task-2", "9"), merged("9", 4), done("task-2"), opened("task-3")],
        vec![opened("task-4"), claimed("task-4"), linked("task-4", "10"), merged_outside("10", 2), done("task-4")],
        vec![escaped("gh is down"), escaped("gh is down"), escaped("hook too strict")],
        vec![Event::Denied { attempt: Attempt::AgentStart }],
    ]
    .concat();
    let (reply, events) = call(&mut given(history, Vec::new()), &["status"], "");
    assert_eq!(events, Vec::new());
    let expected = "open tasks
  task-1 claimed by alice: Fix the hook
  task-3 open: Fix the hook
live leases
  lease-1 for task-1
open escapes: 3
  2x gh is down
  1x hook too strict
P1 agent starts through meta: 1 of 2
P1 merges through meta: 1 of 2
P1 merged outside meta: 1 (0 missing from the log)
P2 open escapes: 3; repeats of an earlier reason: 1
P6 CI runs per merged PR: 6 runs over 2 merged PRs";
    assert_eq!(reply, said(expected));
}

#[test]
fn hook_allows_an_agent_whose_prompt_carries_a_live_unused_lease_and_uses_it_up() {
    let (reply, events) = hook(claimed_task(), &agent("Work on task-1 under lease-1."));
    assert_eq!(reply, Answer { text: ALLOW.to_owned(), code: Code::Done });
    assert_eq!(events, vec![Event::LeaseUsed { lease: id("lease-1") }]);
}

#[test]
fn hook_denies_an_agent_with_no_unknown_used_or_dead_lease_and_says_how_to_get_one() {
    let used = [claimed_task(), vec![Event::LeaseUsed { lease: id("lease-1") }]].concat();
    let dead = [linked_task(), vec![merged("7", 1), done("task-1")]].concat();
    let cases = [
        (claimed_task(), "Work on task-1.", "the Agent prompt carries no lease id."),
        (claimed_task(), "Work under lease-9.", "lease-9 is unknown."),
        (used, "Work under lease-1.", "lease-1 is already used; each lease starts one agent."),
        (dead, "Work under lease-1.", "lease-1 is not live: task-1 is done."),
    ];
    cases.into_iter().for_each(|(events, prompt, why)| {
        let (reply, added) = hook(events, &agent(prompt));
        assert_eq!(added, vec![Event::Denied { attempt: Attempt::AgentStart }]);
        assert_eq!(reply, Answer { text: no_lease(why), code: Code::Done });
    });
}

#[test]
fn hook_gates_the_task_tool_like_the_agent_tool() {
    let task = |prompt: &str| json_input("Task", "prompt", prompt);
    let (reply, events) = hook(claimed_task(), &task("Work under lease-1."));
    assert_eq!((reply.text.as_str(), events), (ALLOW, vec![Event::LeaseUsed { lease: id("lease-1") }]));
    let (reply, events) = hook(claimed_task(), &task("Work on task-1."));
    assert_eq!(events, vec![Event::Denied { attempt: Attempt::AgentStart }]);
    assert_eq!(reply.text, no_lease("the Agent prompt carries no lease id."));
}

#[test]
fn hook_reads_tool_input_only_for_gated_tools() {
    let others = [
        r#"{"tool_name":"Edit","tool_input":{"file_path":"a.rs","prompt":5,"command":["x"]}}"#,
        r#"{"tool_name":"Read"}"#,
        r#"{"tool_name":"WebFetch","tool_input":null}"#,
        r#"{"tool_name":"Bash","tool_input":{"command":"gh pr merge 7 --squash"}}"#,
        r#"{"tool_name":"Bash","tool_input":{"command":"git push origin main"}}"#,
        r#"{"tool_name":"Bash","tool_input":{"command":["gh","pr","merge"]}}"#,
        r#"{"tool_name":"Bash","tool_input":{}}"#,
        r#"{"tool_name":"Bash"}"#,
    ];
    others.into_iter().for_each(|input| {
        assert_eq!(hook(claimed_task(), input), (Answer { text: ALLOW.to_owned(), code: Code::Done }, Vec::new()));
    });
}

#[test]
fn hook_denies_input_that_cannot_be_parsed() {
    let inputs = [
        ("", "EOF while parsing a value at line 1 column 0"),
        ("not json", "expected ident at line 1 column 2"),
        ("{}", "missing field `tool_name` at line 1 column 2"),
        (r#"{"tool_name":"Agent"}"#, "Agent input: invalid type: null, expected struct AgentInput"),
        (
            r#"{"tool_name":"Task","tool_input":{"prompt":5}}"#,
            "Task input: invalid type: integer `5`, expected a string",
        ),
    ];
    inputs.into_iter().for_each(|(input, why)| {
        let (reply, events) = hook(claimed_task(), input);
        assert_eq!(events, Vec::new());
        assert_eq!(reply.text, deny(&format!("hook input cannot be parsed ({why}); denied, fail closed.")));
    });
}

fn json_input(tool: &str, field: &str, value: &str) -> String {
    serde_json::json!({ "tool_name": tool, "tool_input": { field: value } }).to_string()
}

#[test]
fn merge_reads_required_checks_from_recorded_gh_json_and_ignores_unrequired_ones() {
    let context =
        |state: &str| serde_json::json!({ "__typename": "StatusContext", "context": REQUIRED, "state": state });
    let lint =
        serde_json::json!({ "__typename": "CheckRun", "name": "lint", "status": "COMPLETED", "conclusion": "FAILURE" });
    let green = Gh { rollup: vec![context("SUCCESS"), lint.clone()], ..gh(PINNED, &[]) }.replies();
    let mut ports = given(linked_task(), vec![("7", green)]);
    assert_eq!(call(&mut ports, &["merge", "7"], "").1, vec![merged("7", 3), done("task-1")]);
    let red = Gh { rollup: vec![context("ERROR"), lint], ..gh(PINNED, &[]) }.replies();
    let (reply, events) = call(&mut given(linked_task(), vec![("7", red)]), &["merge", "7"], "");
    assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
    let red_gate = "#7 cannot merge:\nred check gates on aaaaaaa. Next: fix it, push, then meta link task-1 7.";
    assert_eq!(reply, refused(red_gate));
}

#[test]
fn merge_reports_a_failed_protection_lookup_or_unreadable_gh_json_as_a_port_failure() {
    let failed = Gh { required: Err("gh api: HTTP 404"), ..gh(PINNED, &[Outcome::Green]) }.replies();
    let garbled = GhReplies { view: "{".to_owned(), ..pull(PINNED, &[Outcome::Green]) };
    let cases = [
        (failed, "error: gh api: HTTP 404\n"),
        (garbled, "error: gh pr view: EOF while parsing an object at line 1 column 1\n"),
    ];
    cases.into_iter().for_each(|(replies, text)| {
        let mut ports = given(linked_task(), vec![("7", replies)]);
        let (reply, events) = call(&mut ports, &["merge", "7"], "");
        assert_eq!((reply, events), (Answer { text: text.to_owned(), code: Code::Failed }, Vec::new()));
        assert!(ports.github.merged.is_empty());
    });
}

#[test]
fn merge_records_a_pr_already_merged_at_the_pinned_head_without_merging_again() {
    let landed = Gh { state: "MERGED", ..gh(PINNED, &[Outcome::Green]) }.replies();
    let mut ports = given(linked_task(), vec![("7", landed)]);
    let (reply, events) = call(&mut ports, &["merge", "7"], "");
    assert_eq!((reply.code, events), (Code::Done, vec![merged_outside("7", 3), done("task-1")]));
    assert!(ports.github.merged.is_empty());
}

#[test]
fn link_refuses_another_branch_another_base_a_pr_linked_elsewhere_or_an_unclaimed_task() {
    let elsewhere = [claimed_task(), vec![opened("task-2"), claimed("task-2"), linked("task-2", "7")]].concat();
    let cases = [
        (
            claimed_task(),
            Gh { branch: "feature", ..gh(PINNED, &[]) },
            "task-1 cannot link #7:\n#7 comes from branch feature, not task-1. Next: push the work to branch task-1.",
        ),
        (
            claimed_task(),
            Gh { base: "dev", ..gh(PINNED, &[]) },
            "task-1 cannot link #7:\n#7 targets dev, not main. Next: retarget it to main.",
        ),
        (elsewhere, gh(PINNED, &[]), "#7 is already linked to task-2."),
        (
            vec![opened("task-1")],
            gh(PINNED, &[]),
            "task-1 open: Fix the hook; only a claimed task can be linked. Next: meta claim task-1 --agent <name>.",
        ),
    ];
    cases.into_iter().for_each(|(events, replies, why)| {
        let (reply, added) = call(&mut given(events, vec![("7", replies.replies())]), &["link", "task-1", "7"], "");
        assert_eq!((reply, added), (refused(why), Vec::new()));
    });
}

#[test]
fn parallel_agent_starts_through_the_jsonl_log_spend_one_lease_once() {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("lease-race-{nanos}.jsonl"));
    let at = Timestamp::from_unix_seconds(1);
    let seed: Vec<Record> = claimed_task().into_iter().map(|event| Record { at, event }).collect();
    meta_ports::Log::append(&mut JsonlLog::at(path.clone()), &seed).unwrap();
    let start = Call::PreToolUse(agent("Work under lease-1."));
    let replies: Vec<Answer> = std::thread::scope(|scope| {
        let racers: Vec<_> = (0..8)
            .map(|_| {
                scope.spawn(|| {
                    let Ports { git, github, clock, .. } = given(Vec::new(), Vec::new());
                    let mut ports = Ports { log: JsonlLog::at(path.clone()), git, github, clock };
                    answer(start.clone(), &mut ports)
                })
            })
            .collect();
        racers.into_iter().map(|racer| racer.join().unwrap()).collect()
    });
    std::fs::remove_file(&path).unwrap();
    assert_eq!(replies.iter().filter(|reply| reply.text == ALLOW).count(), 1);
}

#[test]
fn unknown_or_malformed_commands_answer_with_usage() {
    const USAGE: &str = "usage:
  meta open <title>
  meta claim <task> --agent <name>
  meta link <task> <pr>
  meta merge <pr>
  meta escape <reason>
  meta status
  meta hook pre-tool-use < input
";
    let commands: [(&[&str], &str); 7] = [
        (&[], "unknown command"),
        (&["frobnicate"], "unknown command"),
        (&["claim", "task-1"], "unknown command"),
        (&["claim", "one", "--agent", "alice"], "\"one\" is not a task-<number> id"),
        (&["merge", "seven"], "\"seven\" is not a PR number"),
        (&["open"], "title is empty"),
        (&["hook", "post-tool-use"], "unknown command"),
    ];
    commands.into_iter().for_each(|(args, why)| {
        let words: Vec<String> = args.iter().map(|word| (*word).to_owned()).collect();
        let refused = Call::parse(&words, String::new).unwrap_err();
        assert_eq!(refused, Answer { text: format!("{why}\n{USAGE}"), code: Code::Failed }, "{args:?}");
    });
}

#[test]
fn hook_allows_bash_without_touching_the_log_even_when_the_log_is_corrupt_or_cannot_be_opened() {
    let bash = |command: &str| json_input("Bash", "command", command);
    let corrupt = || given(vec![claimed("task-1")], Vec::new());
    let unopenable = || {
        let mut ports = given(claimed_task(), Vec::new());
        ports.log.broken = Some(PortError::new("cannot open the log"));
        ports
    };
    ["gh pr merge 7", "git push origin main", "claude -p hi", "meta status"].into_iter().for_each(|command| {
        let allowed = Answer { text: ALLOW.to_owned(), code: Code::Done };
        assert_eq!(call(&mut corrupt(), &["hook", "pre-tool-use"], &bash(command)), (allowed.clone(), Vec::new()));
        assert_eq!(call(&mut unopenable(), &["hook", "pre-tool-use"], &bash(command)).0, allowed, "{command}");
    });
}

#[test]
fn a_corrupt_or_unopenable_log_still_records_escapes_but_denies_agent_starts() {
    let corrupt = || vec![claimed("task-1")];
    let (reply, events) = call(&mut given(corrupt(), Vec::new()), &["escape", "log", "is", "corrupt"], "");
    let recorded = "escape recorded: log is corrupt\nopen escapes: 1\nthis reason repeated: 0";
    assert_eq!((reply, events), (said(recorded), vec![escaped("log is corrupt")]));
    let broken = "memory log record 1 does not fit the log before it: task-1 is unknown. Next: meta escape <reason>, then repair memory log.";
    let (reply, events) = hook(corrupt(), &agent("Work under lease-1."));
    assert_eq!(events, Vec::new());
    assert_eq!(reply.text, deny(&format!("meta is broken ({broken}); denied, fail closed.")));
    let (reply, events) = call(&mut given(corrupt(), Vec::new()), &["status"], "");
    assert_eq!((reply, events), (Answer { text: format!("error: {broken}\n"), code: Code::Failed }, Vec::new()));
    let mut unopenable = given(claimed_task(), Vec::new());
    unopenable.log.broken = Some(PortError::new("cannot open the log"));
    let (reply, _) = call(&mut unopenable, &["hook", "pre-tool-use"], &agent("Work under lease-1."));
    assert_eq!(reply.text, deny("meta is broken (cannot open the log); denied, fail closed."));
}

fn listed(prs: &[u64]) -> String {
    let rows: Vec<_> = prs.iter().map(|number| serde_json::json!({ "number": number, "title": "x" })).collect();
    serde_json::Value::Array(rows).to_string()
}

#[test]
fn status_counts_a_pr_merged_through_meta_and_logged_as_through_meta_only() {
    let history = [linked_task(), vec![merged("7", 1), done("task-1")]].concat();
    let mut ports = given(history, Vec::new());
    ports.github.listed = Some(Ok(listed(&[7])));
    let (reply, events) = call(&mut ports, &["status"], "");
    assert_eq!(events, Vec::new());
    let expected = "open tasks
  (none)
live leases
  (none)
open escapes: 0
  (none)
P1 agent starts through meta: 0 of 0
P1 merges through meta: 1 of 1
P1 merged outside meta: 0 (0 missing from the log)
P2 open escapes: 0; repeats of an earlier reason: 0
P6 CI runs per merged PR: 1 runs over 1 merged PRs";
    assert_eq!(reply, said(expected));
}

#[test]
fn status_counts_a_pr_merged_on_github_but_missing_from_the_log_as_merged_outside_meta() {
    let history = [linked_task(), vec![merged("7", 1), done("task-1")]].concat();
    let mut ports = given(history, Vec::new());
    ports.github.listed = Some(Ok(listed(&[7, 12, 13])));
    let (reply, events) = call(&mut ports, &["status"], "");
    let expected = "open tasks
  (none)
live leases
  (none)
open escapes: 0
  (none)
P1 agent starts through meta: 0 of 0
P1 merges through meta: 1 of 3
P1 merged outside meta: 2 (2 missing from the log)
P2 open escapes: 0; repeats of an earlier reason: 0
P6 CI runs per merged PR: 1 runs over 1 merged PRs";
    assert_eq!((reply, events), (said(expected), Vec::new()));
}

#[test]
fn status_reports_the_reconciliation_as_unavailable_when_the_merged_list_cannot_be_read() {
    let failures = [
        (Some(Err(PortError::new("gh is down"))), "gh is down"),
        (Some(Ok("not json".to_owned())), "gh pr list merged: expected ident at line 1 column 2"),
        (None, "no recorded merged list"),
    ];
    failures.into_iter().for_each(|(listed, why)| {
        let mut ports = given(claimed_task(), Vec::new());
        ports.github.listed = listed;
        let (reply, events) = call(&mut ports, &["status"], "");
        let expected = format!(
            "open tasks
  task-1 claimed by alice: Fix the hook
live leases
  lease-1 for task-1
open escapes: 0
  (none)
P1 agent starts through meta: 0 of 0
P1 merges: unavailable, GitHub merged list unreadable ({why}); logged: 0 through meta, 0 outside
P2 open escapes: 0; repeats of an earlier reason: 0
P6 CI runs per merged PR: 0 runs over 0 merged PRs"
        );
        assert_eq!((reply, events), (said(&expected), Vec::new()));
    });
}
