use meta_cli::{Answer, Call, Code, answer};
use meta_core::{
    Attempt, Checked, Claimant, Event, Measure, Outcome, PrNumber, Record, Route, RunCount, SelfFile, Sha, Timestamp,
};
use meta_ports::memory::{FakeClock, MemoryLog, RecordedGit, RecordedGitHub};
use meta_ports::real::JsonlLog;
use meta_ports::{Clock, GhReplies, Log, PortError, Ports};
use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::str::FromStr;

const RULES: &str = include_str!("fixtures/rules.toml");
const KINDS: &str = include_str!("fixtures/escapes.toml");
const PREDICTIONS: &str = include_str!("fixtures/predictions.toml");
const ROLES: &str = include_str!("fixtures/roles.toml");
const ORIGIN: &str = "https://github.com/Studio-Terza/meta-dev.git";
const REPO: &str = "Studio-Terza/meta-dev";
const PINNED: &str = "aaaaaaa";
const MOVED: &str = "bbbbbbb";
const REQUIRED: &str = "gates";
const DENY: &str = r#""permissionDecision":"deny""#;
const ALLOW: &str = "{}\n";
const GET_LEASE: &str = "meta claim <task> --agent <name>";

type Memory = Ports<MemoryLog, RecordedGit, RecordedGitHub, FakeClock>;

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
    Event::Claimed { task: id(task), agent: Claimant::Agent(id("alice")) }
}

fn leased(lease: &str, task: &str) -> Event {
    Event::Leased { lease: id(lease), task: id(task), deadline: None }
}

/// A lease from this change on: its deadline in unix seconds.
fn leased_until(lease: &str, task: &str, deadline: u64) -> Event {
    Event::Leased { lease: id(lease), task: id(task), deadline: Some(Timestamp::from_unix_seconds(deadline)) }
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

fn escaped(kind: &str, reason: &str) -> Event {
    Event::Escaped { kind: id(kind), reason: id(reason), pr: None, checked: Checked::Listed }
}

fn escaped_pr(kind: &str, pr: &str, reason: &str) -> Event {
    Event::Escaped { kind: id(kind), reason: id(reason), pr: Some(id(pr)), checked: Checked::Listed }
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
        git: RecordedGit {
            origin: id(ORIGIN),
            files: self_files(),
            local: self_files(),
            branches: BTreeMap::from([(id("task-1"), id(PINNED))]),
        },
        github: RecordedGitHub {
            earlier: RefCell::default(),
            replies: pulls.into_iter().map(|(pr, replies)| (id::<PrNumber>(pr), replies)).collect(),
            merged: Vec::new(),
            listed: Some(Ok("[]".to_owned())),
        },
        clock: FakeClock::at(Timestamp::from_unix_seconds(2)),
    }
}

fn self_files() -> std::collections::BTreeMap<SelfFile, String> {
    [
        (SelfFile::Rules, RULES),
        (SelfFile::EscapeKinds, KINDS),
        (SelfFile::Predictions, PREDICTIONS),
        (SelfFile::Roles, ROLES),
    ]
    .map(|(file, text)| (file, text.to_owned()))
    .into()
}

fn call(ports: &mut Memory, args: &str, stdin: &str) -> (Answer, Vec<Event>) {
    let before = ports.log.records.len();
    let words: Vec<String> = args.split_whitespace().map(str::to_owned).collect();
    let reply = answer(Call::parse(&words, || stdin.to_owned()).unwrap(), ports);
    let added = ports.log.records[before..].iter().map(|record| record.event.clone()).collect();
    (reply, added)
}

fn parsed(args: &str) -> Call {
    let words: Vec<String> = args.split_whitespace().map(str::to_owned).collect();
    Call::parse(&words, String::new).unwrap()
}

fn hook(events: Vec<Event>, input: &str) -> (Answer, Vec<Event>) {
    call(&mut given(events, Vec::new()), "hook pre-tool-use", input)
}

fn agent(description: &str) -> String {
    serde_json::json!({ "tool_name": "Agent", "tool_input": { "description": description, "prompt": "Work on task-1." } })
        .to_string()
}

fn quoted_values(key: &str) -> Vec<String> {
    let prefix = format!("{key} = \"");
    RULES
        .lines()
        .filter_map(|line| line.strip_prefix(prefix.as_str()))
        .map(|rest| rest.trim_end_matches('"').to_owned())
        .collect()
}

#[test]
fn open_prints_a_new_task_id_and_records_opened() {
    let (reply, events) = call(&mut given(vec![opened("task-1")], Vec::new()), "open Fix the hook", "");
    assert_eq!(reply, Answer { text: "task-2\n".to_owned(), code: Code::Done });
    assert_eq!(events, vec![opened("task-2")]);
}

#[test]
fn claim_of_an_open_unclaimed_task_leases_and_prints_the_brief_with_every_sourced_rule() {
    let (reply, events) = call(&mut given(vec![opened("task-1")], Vec::new()), "claim task-1 --agent alice", "");
    assert_eq!(events, vec![claimed("task-1"), leased_until("lease-1", "task-1", FIXTURE_WORK_DEADLINE)]);
    assert_eq!(reply.code, Code::Done);
    let rules = [quoted_values("id"), quoted_values("text"), quoted_values("source")].concat();
    assert!(rules.len() >= 3 && rules.iter().all(|rule| reply.text.contains(rule.as_str())), "{}", reply.text);
    let brief = ["task-1: Fix the hook", "lease-1", "setup:", &format!("git clone {ORIGIN} task-1")];
    assert!(brief.iter().all(|line| reply.text.contains(line)), "{}", reply.text);
}

#[test]
fn claim_of_a_claimed_done_or_unknown_task_is_refused_with_the_reason() {
    let finished = [linked_task(), vec![merged("7", 1), done("task-1")]].concat();
    let cases = [
        (claimed_task(), "task-1", "already claimed and lease-1 is unused"),
        (finished, "task-1", "is done"),
        (Vec::new(), "task-9", "unknown"),
    ];
    cases.into_iter().for_each(|(events, task, reason)| {
        let (reply, added) = call(&mut given(events, Vec::new()), &format!("claim {task} --agent bob"), "");
        assert_eq!((reply.code, added), (Code::Refused, Vec::new()));
        assert!(reply.text.contains(reason), "{}", reply.text);
    });
    let (reply, _) = call(&mut given(claimed_task(), Vec::new()), "claim task-1 --agent bob", "");
    assert!(
        reply.text.contains("once it is spent, meta claim task-1 --agent <name> issues a fresh lease"),
        "{}",
        reply.text
    );
}

#[test]
fn claim_of_a_claimed_task_whose_lease_is_spent_issues_a_fresh_lease() {
    let spent = Event::LeaseUsed { lease: id("lease-1") };
    [claimed_task(), linked_task()].into_iter().for_each(|history| {
        let history = [history, vec![spent.clone()]].concat();
        let (reply, events) = call(&mut given(history, Vec::new()), "claim task-1 --agent bob", "");
        assert_eq!((reply.code, events), (Code::Done, vec![leased_until("lease-2", "task-1", FIXTURE_WORK_DEADLINE)]));
        assert!(reply.text.contains("lease: lease-2") && reply.text.contains("rules:"), "{}", reply.text);
    });
}

#[test]
fn link_of_a_claimed_task_records_linked_with_the_head_pinned() {
    let mut ports = given(claimed_task(), vec![("7", pull(PINNED, &[]))]);
    let (reply, events) = call(&mut ports, "link task-1 7", "");
    assert_eq!(reply.code, Code::Done);
    assert_eq!(events, vec![linked("task-1", "7")]);
}

#[test]
fn merge_of_a_linked_green_unmoved_pr_squash_merges_with_the_head_pinned() {
    let mut ports = given(linked_task(), vec![("7", pull(PINNED, &[Outcome::Green, Outcome::Green]))]);
    let (reply, events) = call(&mut ports, "merge 7", "");
    assert_eq!(reply.code, Code::Done, "{}", reply.text);
    assert_eq!(events, vec![merged("7", 3), done("task-1")]);
    assert_eq!(ports.github.merged, vec![(id::<PrNumber>("7"), id::<Sha>(PINNED))]);
}

#[test]
fn merge_of_an_unlinked_red_missing_or_moved_pr_is_refused_naming_which_with_the_next_step() {
    let cases = [
        ("8", pull(PINNED, &[Outcome::Green]), "#8 is unlinked", "meta link <task> 8"),
        ("7", pull(PINNED, &[Outcome::Red]), "red check gates", "meta link task-1 7"),
        ("7", pull(PINNED, &[]), "missing check gates", "meta merge 7 again"),
        ("7", pull(PINNED, &[Outcome::Pending]), "missing check gates", "wait for CI"),
        ("7", pull(PINNED, &[Outcome::Green, Outcome::Red]), "red check gates", "meta link task-1 7"),
        ("7", pull(PINNED, &[Outcome::Red, Outcome::Green]), "red check gates", "meta link task-1 7"),
        ("7", pull(PINNED, &[Outcome::Pending, Outcome::Green]), "missing check gates", "wait for CI"),
        ("7", pull(PINNED, &[Outcome::Green, Outcome::Pending]), "missing check gates", "wait for CI"),
        ("7", pull(MOVED, &[Outcome::Green]), "head moved", "meta link task-1 7"),
        (
            "7",
            Gh { required: Ok(Vec::new()), ..gh(PINNED, &[Outcome::Green]) }.replies(),
            "no required checks",
            "Next: require checks on main in branch protection, then meta merge 7 again.",
        ),
        ("7", Gh { state: "CLOSED", ..gh(PINNED, &[Outcome::Green]) }.replies(), "#7 is closed", "reopen it"),
        ("7", Gh { state: "MERGED", ..gh(MOVED, &[Outcome::Green]) }.replies(), "merged outside meta", "head moved"),
    ];
    cases.into_iter().for_each(|(pr, recorded, which, next)| {
        let mut ports = given(linked_task(), vec![(pr, recorded)]);
        let (reply, events) = call(&mut ports, &format!("merge {pr}"), "");
        assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
        assert!(reply.text.contains(which) && reply.text.contains(next), "{}", reply.text);
        assert!(ports.github.merged.is_empty());
    });
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
        vec![
            escaped("gap", "gh is down"),
            escaped("gap", "gh is down again"),
            escaped("outside-merge", "hook too strict"),
        ],
        vec![Event::Denied { attempt: Attempt::AgentStart }],
    ]
    .concat();
    let (reply, events) = call(&mut given(history, Vec::new()), "status", "");
    assert_eq!(events, vec![falsified("P1", Measure::UnrecordedBypasses, "unrecorded bypasses 1")]);
    let expected = [
        "task-1 claimed by alice",
        "task-3 open",
        "lease-1 for task-1",
        "open escapes: 3",
        "escape-1 [gap] gh is down",
        "escape-3 [outside-merge] hook too strict",
        "P1 agent starts through meta: 1; prevented by the hook: 1; bypassed, from outside-start escapes: 0;",
        "P1 merges through meta: 1 of 2",
        "P1 merged outside meta: 1 (0 missing from the log)",
        "P2 open escapes: 3 (0 closed as historical); most repeats of one kind: 1 (gap)",
        "P6 CI runs per merged PR: 6 runs over 2 merged PRs; median 3",
    ];
    assert!(expected.iter().all(|line| reply.text.contains(line)), "{}", reply.text);
    assert!(!reply.text.contains("task-2") && !reply.text.contains("lease-2"), "{}", reply.text);
}

#[test]
fn hook_allows_an_agent_whose_description_is_exactly_a_live_unused_lease_and_uses_it_up() {
    let (reply, events) = hook(claimed_task(), &agent("lease-1"));
    assert_eq!(reply, Answer { text: ALLOW.to_owned(), code: Code::Done });
    assert_eq!(events, vec![Event::LeaseUsed { lease: id("lease-1") }]);
}

#[test]
fn hook_denies_an_agent_with_no_unknown_used_or_dead_lease_and_says_how_to_get_one() {
    let used = [claimed_task(), vec![Event::LeaseUsed { lease: id("lease-1") }]].concat();
    let dead = [linked_task(), vec![merged("7", 1), done("task-1")]].concat();
    let cases = [
        (claimed_task(), "Work on task-1.", "not exactly a lease id"),
        (claimed_task(), "Work under lease-1.", "not exactly a lease id"),
        (claimed_task(), "lease-01", "not exactly a lease id"),
        (claimed_task(), "lease-+1", "not exactly a lease id"),
        (claimed_task(), " lease-1", "not exactly a lease id"),
        (claimed_task(), "lease-9", "lease-9 is unknown"),
        (used, "lease-1", "already used"),
        (dead, "lease-1", "not live"),
    ];
    cases.into_iter().for_each(|(events, description, why)| {
        let (reply, added) = hook(events, &agent(description));
        assert_eq!(added, vec![Event::Denied { attempt: Attempt::AgentStart }]);
        assert!([DENY, why, GET_LEASE].iter().all(|part| reply.text.contains(part)), "{}", reply.text);
    });
}

#[test]
fn hook_gates_the_task_tool_like_the_agent_tool() {
    let task = |description: &str| json_input("Task", "description", description);
    let (reply, events) = hook(claimed_task(), &task("lease-1"));
    assert_eq!((reply.text.as_str(), events), (ALLOW, vec![Event::LeaseUsed { lease: id("lease-1") }]));
    let (reply, events) = hook(claimed_task(), &task("Work on task-1."));
    assert_eq!(events, vec![Event::Denied { attempt: Attempt::AgentStart }]);
    assert!(reply.text.contains(DENY) && reply.text.contains(GET_LEASE), "{}", reply.text);
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
        "",
        "not json",
        "{}",
        r#"{"tool_name":"Agent"}"#,
        r#"{"tool_name":"Task","tool_input":{"prompt":5}}"#,
        r#"{"tool_name":"Agent","tool_input":{"description":5}}"#,
    ];
    inputs.into_iter().for_each(|input| {
        let (reply, events) = hook(claimed_task(), input);
        assert_eq!(events, Vec::new());
        assert!(reply.text.contains(DENY) && reply.text.contains("fail closed"), "{}", reply.text);
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
    assert_eq!(call(&mut ports, "merge 7", "").1, vec![merged("7", 3), done("task-1")]);
    let red = Gh { rollup: vec![context("ERROR"), lint], ..gh(PINNED, &[]) }.replies();
    let (reply, events) = call(&mut given(linked_task(), vec![("7", red)]), "merge 7", "");
    assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
    assert!(reply.text.contains("red check gates") && !reply.text.contains("lint"), "{}", reply.text);
}

#[test]
fn merge_reports_a_failed_protection_lookup_or_unreadable_gh_json_as_a_port_failure() {
    let failed = Gh { required: Err("gh api: HTTP 404"), ..gh(PINNED, &[Outcome::Green]) }.replies();
    let garbled = GhReplies { view: "{".to_owned(), ..pull(PINNED, &[Outcome::Green]) };
    [(failed, "HTTP 404"), (garbled, "gh pr view")].into_iter().for_each(|(replies, why)| {
        let mut ports = given(linked_task(), vec![("7", replies)]);
        let (reply, events) = call(&mut ports, "merge 7", "");
        assert_eq!((reply.code, events), (Code::Failed, Vec::new()));
        assert!(reply.text.starts_with("error:") && reply.text.contains(why), "{}", reply.text);
        assert!(ports.github.merged.is_empty());
    });
}

#[test]
fn merge_records_a_pr_already_merged_at_the_pinned_head_without_merging_again() {
    let landed = Gh { state: "MERGED", ..gh(PINNED, &[Outcome::Green]) }.replies();
    let mut ports = given(linked_task(), vec![("7", landed)]);
    let (reply, events) = call(&mut ports, "merge 7", "");
    assert_eq!((reply.code, events), (Code::Done, vec![merged_outside("7", 3), done("task-1")]));
    assert!(ports.github.merged.is_empty());
}

#[test]
fn link_waits_while_github_still_reports_the_previous_head_then_pins_the_pushed_commit() {
    let mut ports = given(claimed_task(), vec![("7", pull(PINNED, &[]))]);
    ports.git.branches.insert(id("task-1"), id(MOVED));
    let lagging = VecDeque::from([pull(PINNED, &[]), pull(PINNED, &[]), pull(MOVED, &[])]);
    ports.github.earlier.borrow_mut().insert(id("7"), lagging);
    let start = ports.clock.now();
    let (reply, events) = call(&mut ports, "link task-1 7", "");
    assert_eq!(reply.code, Code::Done, "{}", reply.text);
    assert_eq!(events, vec![Event::Linked { task: id("task-1"), pr: id("7"), pin: id(MOVED) }]);
    assert!(ports.github.earlier.borrow()[&id::<PrNumber>("7")].is_empty());
    assert!(ports.clock.now() > start && ports.clock.now() < start.advanced(60));
}

#[test]
fn link_refuses_naming_both_commits_when_github_never_reports_the_pushed_head_before_the_deadline() {
    let mut ports = given(claimed_task(), vec![("7", pull(PINNED, &[]))]);
    ports.git.branches.insert(id("task-1"), id(MOVED));
    let start = ports.clock.now();
    let (reply, events) = call(&mut ports, "link task-1 7", "");
    assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
    for why in [PINNED, MOVED, "push has not landed", "git push origin task-1"] {
        assert!(reply.text.contains(why), "{}", reply.text);
    }
    assert_eq!(ports.clock.now(), start.advanced(60));
}

#[test]
fn link_refuses_with_the_next_step_when_there_is_no_local_branch_for_the_task() {
    let mut ports = given(claimed_task(), vec![("7", pull(PINNED, &[]))]);
    ports.git.branches.clear();
    let start = ports.clock.now();
    let (reply, events) = call(&mut ports, "link task-1 7", "");
    assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
    assert!(reply.text.contains("no local branch task-1"), "{}", reply.text);
    assert!(reply.text.contains("git push origin task-1"), "{}", reply.text);
    assert_eq!(ports.clock.now(), start);
}

#[test]
fn link_refuses_another_branch_another_base_a_pr_linked_elsewhere_or_an_unclaimed_task() {
    let elsewhere = [claimed_task(), vec![opened("task-2"), claimed("task-2"), linked("task-2", "7")]].concat();
    let cases = [
        (claimed_task(), Gh { branch: "feature", ..gh(PINNED, &[]) }, "comes from branch feature, not task-1"),
        (claimed_task(), Gh { base: "dev", ..gh(PINNED, &[]) }, "targets dev, not main"),
        (elsewhere, gh(PINNED, &[]), "#7 is already linked to task-2"),
        (vec![opened("task-1")], gh(PINNED, &[]), "only a claimed task can be linked"),
    ];
    cases.into_iter().for_each(|(events, replies, why)| {
        let (reply, added) = call(&mut given(events, vec![("7", replies.replies())]), "link task-1 7", "");
        assert_eq!((reply.code, added), (Code::Refused, Vec::new()));
        assert!(reply.text.contains(why), "{}", reply.text);
    });
}

#[test]
fn parallel_agent_starts_through_the_jsonl_log_spend_one_lease_once() {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("lease-race-{nanos}.jsonl"));
    let at = Timestamp::from_unix_seconds(1);
    let seed: Vec<Record> = claimed_task().into_iter().map(|event| Record { at, event }).collect();
    JsonlLog::at(path.clone()).transact(|_| (seed, ())).unwrap();
    let start = Call::Hook(meta_core::Phase::Before, agent("lease-1"));
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
    [
        "",
        "frobnicate",
        "claim task-1",
        "claim one --agent alice",
        "merge seven",
        "open",
        "hook stop",
        "escape gap",
        "escape gap --pr seven merged",
        "resolve escape-1",
        "resolve escape-1 --historical",
        "resolve escape-1 --historical --correcting",
        "verdict task-1 maybe",
        "retire task-1",
        "claim task-1 --self bob",
    ]
    .into_iter()
    .for_each(|args| {
        let words: Vec<String> = args.split_whitespace().map(str::to_owned).collect();
        let refused = Call::parse(&words, String::new).unwrap_err();
        assert_eq!(refused.code, Code::Failed);
        assert!(refused.text.contains("usage:"), "{}", refused.text);
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
        assert_eq!(call(&mut corrupt(), "hook pre-tool-use", &bash(command)), (allowed.clone(), Vec::new()));
        assert_eq!(call(&mut unopenable(), "hook pre-tool-use", &bash(command)).0, allowed, "{command}");
    });
}

#[test]
fn a_corrupt_or_unopenable_log_still_records_escapes_but_denies_agent_starts() {
    let corrupt = || vec![claimed("task-1")];
    let (reply, events) = call(&mut given(corrupt(), Vec::new()), "escape gap log is corrupt", "");
    assert_eq!((reply.code, events), (Code::Done, vec![escaped("gap", "log is corrupt")]));
    let repair = "Next: meta escape <kind> <reason>, then repair memory log";
    let (reply, events) = hook(corrupt(), &agent("lease-1"));
    assert_eq!(events, Vec::new());
    assert!(reply.text.contains(DENY) && reply.text.contains("fail closed"), "{}", reply.text);
    assert!(reply.text.contains(repair), "{}", reply.text);
    let (reply, events) = call(&mut given(corrupt(), Vec::new()), "status", "");
    assert_eq!((reply.code, events), (Code::Failed, Vec::new()));
    assert!(reply.text.contains("memory log record 1") && reply.text.contains(repair), "{}", reply.text);
    let mut unopenable = given(claimed_task(), Vec::new());
    unopenable.log.broken = Some(PortError::new("cannot open the log"));
    let (reply, _) = call(&mut unopenable, "hook pre-tool-use", &agent("lease-1"));
    assert!(reply.text.contains(DENY) && reply.text.contains("cannot open the log"), "{}", reply.text);
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
    let (reply, events) = call(&mut ports, "status", "");
    assert_eq!(events, Vec::new());
    let expected = ["P1 merges through meta: 1 of 1", "P1 merged outside meta: 0 (0 missing from the log)"];
    assert!(expected.iter().all(|line| reply.text.contains(line)), "{}", reply.text);
}

#[test]
fn status_counts_a_pr_merged_on_github_but_missing_from_the_log_as_merged_outside_meta() {
    let history = [linked_task(), vec![merged("7", 1), done("task-1")]].concat();
    let mut ports = given(history, Vec::new());
    ports.github.listed = Some(Ok(listed(&[7, 12, 13])));
    let (reply, events) = call(&mut ports, "status", "");
    let latched = vec![falsified("P1", Measure::UnrecordedBypasses, "unrecorded bypasses 2")];
    assert_eq!((reply.code, events), (Code::Done, latched));
    let expected = ["P1 merges through meta: 1 of 3", "P1 merged outside meta: 2 (2 missing from the log)"];
    assert!(expected.iter().all(|line| reply.text.contains(line)), "{}", reply.text);
}

#[test]
fn status_reports_the_reconciliation_as_unavailable_when_the_merged_list_cannot_be_read() {
    let failures = [Some(Err(PortError::new("gh is down"))), Some(Ok("not json".to_owned())), None];
    failures.into_iter().for_each(|listed| {
        let mut ports = given(claimed_task(), Vec::new());
        ports.github.listed = listed;
        let (reply, events) = call(&mut ports, "status", "");
        assert_eq!((reply.code, events), (Code::Done, Vec::new()));
        assert!(reply.text.contains("P1 merges: unavailable, GitHub merged list unreadable"), "{}", reply.text);
        assert!(reply.text.contains("open tasks") && reply.text.contains("P2 open escapes"), "{}", reply.text);
    });
}

#[test]
fn status_reports_the_reconciliation_as_unavailable_when_the_merged_list_may_be_truncated() {
    let full = |count: u64| listed(&(1..=count).collect::<Vec<_>>());
    let mut ports = given(claimed_task(), Vec::new());
    ports.github.listed = Some(Ok(full(1001)));
    let (reply, events) = call(&mut ports, "status", "");
    assert_eq!((reply.code, events), (Code::Done, Vec::new()));
    assert!(reply.text.contains("P1 merges: unavailable") && reply.text.contains("truncated"), "{}", reply.text);
    let mut ports = given(claimed_task(), Vec::new());
    ports.github.listed = Some(Ok(full(1000)));
    let (reply, _) = call(&mut ports, "status", "");
    assert!(reply.text.contains("P1 merged outside meta: 1000"), "{}", reply.text);
}

// Stage 1 (docs/stage1.md). Each test below is one row of its behaviour tables.

fn used(lease: &str) -> Event {
    Event::LeaseUsed { lease: id(lease) }
}

fn resolved(escape: &str, task: &str) -> Event {
    Event::Resolved { escape: id(escape), task: id(task) }
}

fn historical(escape: &str, reason: &str) -> Event {
    Event::ResolvedHistorical { escape: id(escape), reason: id(reason) }
}

fn corrected(escape: &str, reason: &str) -> Event {
    Event::Corrected { escape: id(escape), reason: id(reason) }
}

fn self_claimed(task: &str) -> Event {
    Event::Claimed { task: id(task), agent: Claimant::Lead }
}

fn review_leased(lease: &str, head: &str) -> Event {
    Event::ReviewLeased { lease: id(lease), task: id("task-1"), pr: id("7"), head: id(head), deadline: None }
}

fn review_leased_until(lease: &str, head: &str, deadline: u64) -> Event {
    let deadline = Some(Timestamp::from_unix_seconds(deadline));
    Event::ReviewLeased { lease: id(lease), task: id("task-1"), pr: id("7"), head: id(head), deadline }
}

fn finished_task() -> Vec<Event> {
    [linked_task(), vec![merged("7", 1), done("task-1")]].concat()
}

const AGENT_ID: &str = "a4d2c8f1e0b3a297";

fn send(to: &str, message: serde_json::Value, summary: Option<&str>) -> String {
    let input = match summary {
        Some(summary) => serde_json::json!({ "to": to, "message": message, "summary": summary }),
        None => serde_json::json!({ "to": to, "message": message }),
    };
    serde_json::json!({ "tool_name": "SendMessage", "tool_input": input }).to_string()
}

fn started(lease: &str, agent: &str) -> Event {
    Event::Started { lease: id(lease), agent: id(agent), deadline: None }
}

/// A finished or backgrounded Agent call as the PostToolUse hook gets it (Claude Code hooks docs:
/// tool_response.agentId and tool_response.status).
fn agent_ran(tool: &str, description: &str, agent: &str) -> String {
    serde_json::json!({
        "hook_event_name": "PostToolUse",
        "tool_name": tool,
        "tool_input": { "description": description, "prompt": "Work on task-1." },
        "tool_response": { "status": "async_launched", "agentId": agent, "description": description },
    })
    .to_string()
}

fn after(events: Vec<Event>, input: &str) -> (Answer, Vec<Event>) {
    call(&mut given(events, Vec::new()), "hook post-tool-use", input)
}

fn bound() -> Vec<Event> {
    [claimed_task(), vec![used("lease-1"), started("lease-1", AGENT_ID)]].concat()
}

fn status_of(events: Vec<Event>) -> String {
    let (reply, added) = call(&mut given(events, Vec::new()), "status", "");
    assert_eq!(reply.code, Code::Done, "{}", reply.text);
    assert!(added.iter().all(|event| matches!(event, Event::Falsified { .. })), "{added:?}");
    reply.text
}

#[test]
fn escape_of_a_listed_kind_records_escaped_and_prints_its_id_the_open_count_and_the_repeats_of_that_kind() {
    let history = [
        finished_task(),
        vec![escaped("gap", "no retire"), escaped("outside-merge", "merged by hand"), escaped("gap", "no review")],
        vec![resolved("escape-1", "task-1")],
    ]
    .concat();
    let (reply, events) = call(&mut given(history, Vec::new()), "escape gap no self-claim", "");
    assert_eq!((reply.code, events), (Code::Done, vec![escaped("gap", "no self-claim")]));
    let expected = ["escape-4 recorded: [gap] no self-claim", "open escapes: 3", "repeats of kind gap: 2"];
    assert!(expected.iter().all(|line| reply.text.contains(line)), "{}", reply.text);
}

#[test]
fn escape_of_an_unknown_kind_is_refused_and_lists_the_kinds() {
    ["hook too strict", "unclassified old text"].into_iter().for_each(|args| {
        let (reply, events) = call(&mut given(Vec::new(), Vec::new()), &format!("escape {args}"), "");
        assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
        let listed = ["is not an escape kind", "- gap: real work needed", "source: fixture: gap", "- outside-merge:"];
        assert!(listed.iter().all(|part| reply.text.contains(part)), "{}", reply.text);
    });
}

#[test]
fn resolve_against_a_done_task_records_resolved_and_the_escape_no_longer_counts_as_open() {
    let history = [finished_task(), vec![escaped("gap", "no retire"), escaped("gap", "no review")]].concat();
    let mut ports = given(history, Vec::new());
    let (reply, events) = call(&mut ports, "resolve escape-1 task-1", "");
    assert_eq!((reply.code, events), (Code::Done, vec![resolved("escape-1", "task-1")]));
    let (reply, _) = call(&mut ports, "status", "");
    assert!(reply.text.contains("open escapes: 1\n  escape-2 [gap] no review\n"), "{}", reply.text);
    assert!(reply.text.contains("P2 open escapes: 1 (0 closed as historical);"), "{}", reply.text);
    assert!(!reply.text.contains("escape-1"), "{}", reply.text);
}

#[test]
fn resolve_is_refused_with_the_reason_when_the_task_is_not_done_or_the_escape_is_resolved_or_unknown() {
    let escapes = || vec![escaped("gap", "no retire")];
    let cases = [
        ([claimed_task(), escapes()].concat(), "escape-1 task-1", "only against a done task"),
        (
            [finished_task(), escapes(), vec![resolved("escape-1", "task-1")]].concat(),
            "escape-1 task-1",
            "already resolved",
        ),
        ([finished_task(), escapes()].concat(), "escape-2 task-1", "escape-2 is unknown"),
        ([finished_task(), escapes()].concat(), "escape-0 task-1", "escape-0 is unknown"),
        ([finished_task(), escapes()].concat(), "escape-1 task-9", "task-9 is unknown"),
    ];
    cases.into_iter().for_each(|(history, args, why)| {
        let (reply, events) = call(&mut given(history, Vec::new()), &format!("resolve {args}"), "");
        assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
        assert!(reply.text.contains(why), "{}", reply.text);
    });
}

#[test]
fn escapes_logged_before_kinds_fold_as_unclassified_beside_old_denied_and_lease_used_records() {
    let text = include_str!("fixtures/stage0-log.jsonl");
    let records: Vec<Record> = text.lines().map(|line| serde_json::from_str(line).unwrap()).collect();
    let unclassified = |reason: &str| escaped("unclassified", reason);
    assert_eq!(records[2].event, unclassified("bootstrap: agent started without a lease (meta does not exist yet)"));
    let mut ports = given(Vec::new(), Vec::new());
    ports.log.records = records;
    let (reply, _) = call(&mut ports, "status", "");
    let expected = [
        "escape-1 [unclassified] bootstrap: agent started",
        "escape-2 [unclassified] no retire command",
        "P1 agent starts through meta: 1; prevented by the hook: 1; bypassed, from outside-start escapes: 0;",
        "P2 open escapes: 2 (0 closed as historical); most repeats of one kind: 0",
    ];
    assert!(expected.iter().all(|line| reply.text.contains(line)), "{}", reply.text);
    let (reply, events) = call(&mut ports, "resolve escape-1 task-1", "");
    assert_eq!((reply.code, events), (Code::Done, vec![resolved("escape-1", "task-1")]));
}

#[test]
fn claim_self_of_an_open_task_records_claimed_by_the_lead_with_no_lease_and_prints_the_brief() {
    let (reply, events) = call(&mut given(vec![opened("task-1")], Vec::new()), "claim task-1 --self", "");
    assert_eq!((reply.code, events), (Code::Done, vec![self_claimed("task-1")]));
    let brief = ["task-1: Fix the hook", "lease: none", "setup:", "rules:", "fixture: identity"];
    assert!(brief.iter().all(|line| reply.text.contains(line)), "{}", reply.text);
    let status = status_of(vec![opened("task-1"), self_claimed("task-1"), opened("task-2")]);
    let expected = [
        "task-1 claimed by the lead",
        "P1 agent starts through meta: 0; prevented by the hook: 0; bypassed, from outside-start escapes: 0; self-claims, not agent starts: 1",
    ];
    assert!(expected.iter().all(|line| status.contains(line)), "{status}");
    let record = Record { at: Timestamp::from_unix_seconds(1), event: self_claimed("task-1") };
    let line = serde_json::to_string(&record).unwrap();
    assert_eq!(line, r#"{"at":1,"event":"Claimed","task":1}"#);
    assert_eq!(serde_json::from_str::<Record>(&line).unwrap(), record);
}

#[test]
fn claim_self_of_a_task_claimed_by_an_agent_is_refused() {
    let (reply, events) = call(&mut given(claimed_task(), Vec::new()), "claim task-1 --self", "");
    assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
    assert!(reply.text.contains("already claimed by alice"), "{}", reply.text);
}

#[test]
fn retire_of_an_open_or_claimed_task_not_linked_to_an_open_pr_records_retired_and_leaves_the_open_list() {
    let closed = Gh { state: "CLOSED", ..gh(PINNED, &[]) }.replies();
    let cases = [(vec![opened("task-1")], None), (claimed_task(), None), (linked_task(), Some(closed))];
    cases.into_iter().for_each(|(history, pull)| {
        let pulls = pull.into_iter().map(|replies| ("7", replies)).collect();
        let mut ports = given(history, pulls);
        let (reply, events) = call(&mut ports, "retire task-1 moot after #5", "");
        let retired = Event::Retired { task: id("task-1"), reason: id("moot after #5") };
        assert_eq!((reply.code, events), (Code::Done, vec![retired]), "{}", reply.text);
        let (status, _) = call(&mut ports, "status", "");
        assert!(status.text.contains("open tasks\n  (none)\n"), "{}", status.text);
        assert!(status.text.contains("live leases\n  (none)\n"), "{}", status.text);
    });
    let open = given(linked_task(), vec![("7", pull(PINNED, &[]))]);
    let (reply, events) = call(&mut { open }, "retire task-1 moot", "");
    assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
    assert!(reply.text.contains("#7 is linked and open"), "{}", reply.text);
}

#[test]
fn defer_of_an_open_task_records_deferred_and_lists_it_under_deferred() {
    let mut ports = given(vec![opened("task-1")], Vec::new());
    let (reply, events) = call(&mut ports, "defer task-1 after stage 2", "");
    let deferred = Event::Deferred { task: id("task-1"), reason: id("after stage 2") };
    assert_eq!((reply.code, events), (Code::Done, vec![deferred]));
    let (status, _) = call(&mut ports, "status", "");
    assert!(
        status.text.contains("open tasks\n  (none)\ndeferred\n  task-1 deferred (after stage 2)"),
        "{}",
        status.text
    );
    let (reply, events) = call(&mut given(claimed_task(), Vec::new()), "defer task-1 later", "");
    assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
}

#[test]
fn undefer_of_a_deferred_task_records_undeferred_and_opens_it_again() {
    let history = vec![opened("task-1"), Event::Deferred { task: id("task-1"), reason: id("later") }];
    let mut ports = given(history, Vec::new());
    let (reply, events) = call(&mut ports, "undefer task-1", "");
    assert_eq!((reply.code, events), (Code::Done, vec![Event::Undeferred { task: id("task-1") }]));
    let (status, _) = call(&mut ports, "status", "");
    assert!(status.text.contains("open tasks\n  task-1 open") && status.text.contains("deferred\n  (none)"));
    let (reply, events) = call(&mut given(vec![opened("task-1")], Vec::new()), "undefer task-1", "");
    assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
}

#[test]
fn review_of_a_task_linked_to_an_open_pr_leases_and_prints_an_unprimed_brief_with_the_review_rule() {
    let history = [linked_task(), vec![used("lease-1")]].concat();
    let mut ports = given(history, vec![("7", pull(MOVED, &[]))]);
    let (reply, events) = call(&mut ports, "review task-1 --agent rev", "");
    let expected = vec![review_leased_until("lease-2", MOVED, FIXTURE_REVIEW_DEADLINE)];
    assert_eq!((reply.code, events), (Code::Done, expected), "{}", reply.text);
    let brief = ["review: #7 for task-1", "agent: rev", "lease: lease-2", "goal:", "constraints:", "review-gates"];
    assert!(brief.iter().all(|line| reply.text.contains(line)), "{}", reply.text);
    let (status, _) = call(&mut ports, "status", "");
    assert!(status.text.contains("lease-2 for task-1"), "{}", status.text);
    let refusals = [
        (claimed_task(), pull(PINNED, &[]), "only a task linked to an open PR"),
        (linked_task(), Gh { state: "CLOSED", ..gh(PINNED, &[]) }.replies(), "#7 is not open"),
    ];
    refusals.into_iter().for_each(|(history, replies, why)| {
        let (reply, events) = call(&mut given(history, vec![("7", replies)]), "review task-1 --agent rev", "");
        assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
        assert!(reply.text.contains(why), "{}", reply.text);
    });
}

#[test]
fn verdict_after_a_used_review_lease_records_reviewed_and_status_shows_the_latest_verdict_per_linked_pr() {
    let reviewed = |verdict: &str| Event::Reviewed { lease: id("lease-2"), verdict: id(verdict) };
    let (reply, events) = call(&mut given(linked_task(), Vec::new()), "verdict task-1 pass", "");
    assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
    let unused = [linked_task(), vec![review_leased("lease-2", PINNED)]].concat();
    let (reply, _) = call(&mut given(unused.clone(), Vec::new()), "verdict task-1 pass", "");
    assert!(reply.code == Code::Refused && reply.text.contains("awaits a verdict"), "{}", reply.text);
    let mut ports = given([unused, vec![used("lease-2")]].concat(), Vec::new());
    let (reply, events) = call(&mut ports, "verdict task-1 fail", "");
    assert_eq!((reply.code, events), (Code::Done, vec![reviewed("fail")]));
    let (reply, _) = call(&mut ports, "verdict task-1 pass", "");
    assert_eq!(reply.code, Code::Refused);
    let later = vec![
        review_leased("lease-3", MOVED),
        used("lease-3"),
        Event::Reviewed { lease: id("lease-3"), verdict: id("pass") },
    ];
    ports.log.records.extend(later.into_iter().map(|event| Record { at: Timestamp::from_unix_seconds(3), event }));
    let (status, _) = call(&mut ports, "status", "");
    let line = format!("latest review verdict per linked PR\n  #7 task-1: pass at {MOVED}\n");
    assert!(status.text.contains(&line), "{}", status.text);
}

#[test]
fn hook_allows_a_canned_continue_to_the_agent_a_used_lease_started_and_records_resumed() {
    let (reply, events) = hook(bound(), &send(AGENT_ID, serde_json::json!("continue"), Some("lease-1")));
    assert_eq!((reply.text.as_str(), events), (ALLOW, vec![Event::Resumed { lease: id("lease-1") }]));
    let resumed = |times: usize| {
        let resumes = (0..times).map(|_| Event::Resumed { lease: id("lease-1") });
        [bound(), resumes.collect()].concat()
    };
    let flagged = "leases resumed 3 or more times (flagged)\n  lease-1: resumed 3 times\n";
    assert!(status_of(resumed(3)).contains(flagged));
    assert!(status_of(resumed(2)).contains("leases resumed 3 or more times (flagged)\n  (none)\n"));
}

#[test]
fn hook_denies_any_other_send_message_since_new_instructions_need_a_fresh_agent_on_a_fresh_lease() {
    let to = |agent: &str, message: serde_json::Value, summary: Option<&str>| send(agent, message, summary);
    let canned = || serde_json::json!("continue");
    let unbound = [claimed_task(), vec![used("lease-1")]].concat();
    let finished = [finished_task(), vec![used("lease-1"), started("lease-1", AGENT_ID)]].concat();
    let cases = [
        (bound(), to(AGENT_ID, serde_json::json!("continue, then also fix the docs"), Some("lease-1")), "not exactly"),
        (bound(), to(AGENT_ID, serde_json::json!("Continue"), Some("lease-1")), "not exactly"),
        (bound(), to(AGENT_ID, serde_json::json!({ "type": "shutdown_request" }), Some("lease-1")), "not exactly"),
        (bound(), to(AGENT_ID, canned(), None), "not a lease id"),
        (bound(), to(AGENT_ID, canned(), Some("resume lease-1")), "not exactly a lease id"),
        (bound(), to(AGENT_ID, canned(), Some("lease-9")), "lease-9 is unknown"),
        (claimed_task(), to(AGENT_ID, canned(), Some("lease-1")), "lease-1 is unused"),
        (bound(), to("worker", canned(), Some("lease-1")), &format!("lease-1 started {AGENT_ID}, not worker")),
        (bound(), to("", canned(), Some("lease-1")), "agent id is empty"),
        (unbound, to(AGENT_ID, canned(), Some("lease-1")), "no agent is recorded as started on lease-1"),
        (finished, to(AGENT_ID, canned(), Some("lease-1")), "the task of lease-1 is no longer active: task-1 done"),
    ];
    cases.into_iter().for_each(|(history, input, why)| {
        let (reply, events) = hook(history, &input);
        assert_eq!(events, Vec::new());
        let parts = [DENY, why, "new instructions need a fresh agent on a fresh lease"];
        assert!(parts.iter().all(|part| reply.text.contains(part)), "{}", reply.text);
    });
    let (reply, _) = hook(vec![claimed("task-1")], &send(AGENT_ID, serde_json::json!("continue"), Some("lease-1")));
    assert!(reply.text.contains(DENY) && reply.text.contains("fail closed"), "{}", reply.text);
}

#[test]
fn status_shows_each_prediction_with_its_observed_value_its_registered_expectation_and_falsified_when_it_holds() {
    let calm = [finished_task(), vec![escaped("gap", "no retire")]].concat();
    let status = status_of(calm);
    let lines = [
        "P1 The loop is used: UNDECIDED; observed unrecorded bypasses unavailable (agent starts without a lease are not observable expected 100%, except recorded escapes",
        "P2 Gaps get fixed, not bypassed: holds; observed open escapes 1, most repeats of one escape kind 0; \
         expected Each reason fixed before its 3rd repeat; at most 5 open",
        "P6 Little rework: holds; observed median CI runs per merged PR 1; expected Median at most 2",
    ];
    assert!(lines.iter().all(|line| status.contains(line)), "{status}");
    let gaps = (0..4).map(|number| escaped("gap", &format!("gap {number}")));
    let opened_and_merged = (2..=3).flat_map(|task| {
        let (task, pr) = (format!("task-{task}"), format!("{}", task + 6));
        vec![opened(&task), claimed(&task), linked(&task, &pr), merged(&pr, 4), done(&task)]
    });
    let rough = [finished_task(), gaps.collect(), opened_and_merged.collect()].concat();
    let mut ports = given(rough, Vec::new());
    ports.github.listed = Some(Ok(listed(&[1, 7, 8, 9])));
    let (reply, _) = call(&mut ports, "status", "");
    let falsified = [
        "P1 The loop is used: FALSIFIED; observed unrecorded bypasses 1;",
        "P2 Gaps get fixed, not bypassed: FALSIFIED; observed open escapes 4, most repeats of one escape kind 3;",
        "P6 Little rework: FALSIFIED; observed median CI runs per merged PR 4;",
    ];
    assert!(falsified.iter().all(|line| reply.text.contains(line)), "{}", reply.text);
    ports.log.records.push(Record {
        at: Timestamp::from_unix_seconds(3),
        event: escaped_pr("outside-merge", "1", "merged by hand"),
    });
    let (reply, _) = call(&mut ports, "status", "");
    assert!(reply.text.contains("P1 The loop is used: UNDECIDED; observed unrecorded bypasses unavailable (agent starts without a lease are not observable"), "{}", reply.text);
    ports.github.listed = Some(Err(PortError::new("gh is down")));
    let (reply, _) = call(&mut ports, "status", "");
    assert!(
        reply.text.contains("P1 The loop is used: UNDECIDED; observed unrecorded bypasses unavailable"),
        "{}",
        reply.text
    );
}

// Stage 1 review fixes (PR #16 review). Each test below is one row of the behaviour tables.

#[test]
fn status_falsifies_p1_for_each_pr_merged_outside_meta_that_no_outside_merge_escape_names() {
    let outside =
        vec![opened("task-2"), claimed("task-2"), linked("task-2", "8"), merged_outside("8", 1), done("task-2")];
    let history = [finished_task(), outside].concat();
    let p1 = |escapes: Vec<Event>| {
        let mut ports = given([history.clone(), escapes].concat(), Vec::new());
        ports.github.listed = Some(Ok(listed(&[7, 8, 12])));
        call(&mut ports, "status", "").0.text
    };
    let cases = [
        (Vec::new(), "FALSIFIED; observed unrecorded bypasses 2;"),
        (vec![escaped("unclassified", "merged #8 and #12 by hand")], "FALSIFIED; observed unrecorded bypasses 2;"),
        (vec![escaped("gap", "merged #8 by hand"), escaped_pr("gap", "12", "merged by hand")], "bypasses 2;"),
        (vec![escaped_pr("outside-merge", "8", "merged by hand")], "FALSIFIED; observed unrecorded bypasses 1;"),
        (vec![escaped_pr("outside-merge", "8", "once"), escaped_pr("outside-merge", "8", "twice")], "bypasses 1;"),
        (vec![escaped_pr("outside-merge", "99", "merged by hand")], "FALSIFIED; observed unrecorded bypasses 2;"),
        (
            vec![escaped_pr("outside-merge", "8", "by hand"), escaped_pr("outside-merge", "12", "by hand")],
            "P1 The loop is used: UNDECIDED; observed unrecorded bypasses unavailable (agent starts without a lease are not observable",
        ),
    ];
    cases.into_iter().for_each(|(escapes, expected)| {
        let status = p1(escapes);
        assert!(status.contains(expected), "{expected}\n{status}");
    });
    assert!(p1(Vec::new()).contains("P1 The loop is used: FALSIFIED"));
}

#[test]
fn escape_of_a_kind_that_names_a_merged_pr_records_the_pr_and_any_other_kind_refuses_one() {
    let mut ports = given(Vec::new(), Vec::new());
    let (reply, events) = call(&mut ports, "escape outside-merge --pr 12 merged by hand", "");
    assert_eq!((reply.code, events), (Code::Done, vec![escaped_pr("outside-merge", "12", "merged by hand")]));
    assert!(reply.text.contains("escape-1 recorded: [outside-merge] #12 merged by hand"), "{}", reply.text);
    let refusals = [
        ("escape outside-merge merged by hand", "outside-merge names the PR merged outside meta"),
        ("escape gap --pr 12 no retire", "gap names no PR"),
    ];
    refusals.into_iter().for_each(|(args, why)| {
        let (reply, events) = call(&mut given(Vec::new(), Vec::new()), args, "");
        assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
        assert!(reply.text.contains(why), "{}", reply.text);
    });
}

#[test]
fn post_hook_records_the_agent_id_started_on_a_used_lease() {
    let spent = || [claimed_task(), vec![used("lease-1")]].concat();
    ["Agent", "Task"].into_iter().for_each(|tool| {
        let (reply, events) = after(spent(), &agent_ran(tool, "lease-1", AGENT_ID));
        assert_eq!((reply.text.as_str(), events), (ALLOW, vec![started("lease-1", AGENT_ID)]));
    });
    let block = r#""decision":"block""#;
    let cases = [
        (bound(), agent_ran("Agent", "lease-1", "b0b"), format!("lease-1 already started {AGENT_ID}")),
        (claimed_task(), agent_ran("Agent", "lease-1", AGENT_ID), "lease-1 is unused".to_owned()),
        (spent(), agent_ran("Agent", "lease-9", AGENT_ID), "lease-9 is unknown".to_owned()),
        (spent(), agent_ran("Agent", "Work on task-1.", AGENT_ID), "not exactly a lease id".to_owned()),
        (spent(), json_input("Agent", "description", "lease-1"), "fail closed".to_owned()),
        (vec![claimed("task-1")], agent_ran("Agent", "lease-1", AGENT_ID), "fail closed".to_owned()),
    ];
    cases.into_iter().for_each(|(history, input, why)| {
        let (reply, events) = after(history, &input);
        assert_eq!((reply.code, events), (Code::Done, Vec::new()));
        assert!(reply.text.contains(block) && reply.text.contains(why.as_str()), "{}", reply.text);
    });
    let other = r#"{"tool_name":"Bash","tool_input":{"command":"ls"},"tool_response":{}}"#;
    assert_eq!(after(spent(), other), (Answer { text: ALLOW.to_owned(), code: Code::Done }, Vec::new()));
}

#[test]
fn escape_reads_its_kinds_without_git_and_records_an_unchecked_kind_when_none_can_be_read() {
    let mut ports = given(Vec::new(), Vec::new());
    ports.git.files.clear();
    let (reply, events) = call(&mut ports, "escape gap no retire", "");
    assert_eq!((reply.code, events), (Code::Done, vec![escaped("gap", "no retire")]), "{}", reply.text);
    ports.git.local.clear();
    let unchecked = |kind: &str, reason: &str| Event::Escaped {
        kind: id(kind),
        reason: id(reason),
        pr: None,
        checked: Checked::Unchecked,
    };
    let (reply, events) = call(&mut ports, "escape gap no review", "");
    assert_eq!((reply.code, events), (Code::Done, vec![unchecked("gap", "no review")]));
    let why = ["escape-2 recorded: [gap] no review", "kind unchecked: no local self/escapes.toml"];
    assert!(why.iter().all(|part| reply.text.contains(part)), "{}", reply.text);
    let (reply, events) = call(&mut ports, "escape gpa typo kept as given", "");
    assert_eq!((reply.code, events), (Code::Done, vec![unchecked("gpa", "typo kept as given")]));
    let (reply, events) = call(&mut ports, "escape unclassified old text", "");
    assert_eq!((reply.code, events), (Code::Refused, Vec::new()), "{}", reply.text);
    let status = call(&mut ports, "status", "").0.text;
    assert!(status.contains("escape-2 [gap, unchecked] no review"), "{status}");
}

#[test]
fn escape_is_recorded_even_when_the_jsonl_log_does_not_parse() {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("garbled-{nanos}.jsonl"));
    std::fs::write(&path, "not a record\n").unwrap();
    let Ports { git, github, clock, .. } = given(Vec::new(), Vec::new());
    let mut ports = Ports { log: JsonlLog::at(path.clone()), git, github, clock };
    let reply = answer(parsed("escape gap log is garbled"), &mut ports);
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
    assert_eq!(reply.code, Code::Done, "{}", reply.text);
    assert!(reply.text.contains("id and counts unavailable"), "{}", reply.text);
    assert_eq!(text.lines().last(), Some(r#"{"at":2,"event":"Escaped","kind":"gap","reason":"log is garbled"}"#));
}

/// A log whose plain read is stale: only a read inside the transaction sees every record.
struct Lagging(MemoryLog);

impl Log for Lagging {
    fn place(&self) -> String {
        self.0.place()
    }

    fn read(&self) -> Result<Vec<Record>, PortError> {
        Ok(Vec::new())
    }

    fn transact<T>(
        &mut self,
        decide: impl FnOnce(Result<Vec<Record>, PortError>) -> (Vec<Record>, T),
    ) -> Result<T, PortError> {
        self.0.transact(decide)
    }
}

#[test]
fn escape_prints_the_id_it_gets_inside_the_locked_append() {
    let Ports { log, git, github, clock } = given(vec![escaped("gap", "one"), escaped("gap", "two")], Vec::new());
    let mut ports = Ports { log: Lagging(log), git, github, clock };
    let reply = answer(parsed("escape gap three"), &mut ports);
    let expected = ["escape-3 recorded: [gap] three", "open escapes: 3", "repeats of kind gap: 2"];
    assert!(expected.iter().all(|line| reply.text.contains(line)), "{}", reply.text);
}

#[test]
fn status_falsifies_p2_at_the_third_escape_of_one_kind_and_quotes_its_reading_of_reason() {
    let gaps = |count: usize| (0..count).map(|number| escaped("gap", &format!("gap {number}"))).collect::<Vec<_>>();
    let p2 = "P2 Gaps get fixed, not bypassed:";
    let twice = status_of(gaps(2));
    assert!(twice.contains(&format!("{p2} holds; observed open escapes 2, most repeats of one escape kind 1;")));
    let thrice = status_of(gaps(3));
    assert!(thrice.contains(&format!("{p2} FALSIFIED; observed open escapes 3, most repeats of one escape kind 2;")));
    let quoted = "expected Each reason fixed before its 3rd repeat; at most 5 open; \
                  falsified by A reason repeated 3 times, or more than 5 open; \
                  measure: Open escapes; repeats per reason; read reason as escape kind";
    assert!(thrice.contains(quoted), "{thrice}");
}

#[test]
fn status_shows_a_prediction_without_a_falsifier_as_undecided() {
    let mut ports = given(Vec::new(), Vec::new());
    let unfalsifiable = "[[prediction]]\nid = \"P9\"\nhypothesis = \"Untestable\"\nmeasure = \"m\"\n\
                         expected = \"e\"\nfalsified_by = \"f\"\n";
    ports.git.files.insert(SelfFile::Predictions, unfalsifiable.to_owned());
    let status = call(&mut ports, "status", "").0.text;
    assert!(status.contains("P9 Untestable: UNDECIDED; no falsifier registered;"), "{status}");
}

#[test]
fn claim_by_an_agent_of_a_self_claimed_task_is_refused() {
    let self_claimed_task = || vec![opened("task-1"), self_claimed("task-1")];
    let histories = [self_claimed_task(), [self_claimed_task(), vec![linked("task-1", "7")]].concat()];
    histories.into_iter().for_each(|history| {
        let (reply, events) = call(&mut given(history, Vec::new()), "claim task-1 --agent bob", "");
        assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
        assert!(reply.text.contains("self-claimed"), "{}", reply.text);
    });
}

#[test]
fn retire_of_a_deferred_task_is_refused() {
    let history = vec![opened("task-1"), Event::Deferred { task: id("task-1"), reason: id("later") }];
    let (reply, events) = call(&mut given(history, Vec::new()), "retire task-1 moot", "");
    assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
    assert!(reply.text.contains("only an open or claimed task can be retired"), "{}", reply.text);
}

fn falsified(prediction: &str, measure: Measure, observed: &str) -> Event {
    Event::Falsified { prediction: id(prediction), measure, observed: id(observed) }
}

fn unclassified(count: usize) -> Vec<Event> {
    (1..=count).map(|number| escaped("unclassified", &format!("old {number}"))).collect()
}

#[test]
fn status_latches_each_falsification_once_and_shows_it_after_the_reading_recovers() {
    let gaps = |count: usize| (0..count).map(|number| escaped("gap", &format!("gap {number}"))).collect::<Vec<_>>();
    let mut ports = given(gaps(3), Vec::new());
    let (reply, added) = call(&mut ports, "status", "");
    let repeats = "most repeats of one escape kind 2";
    assert_eq!((reply.code, added), (Code::Done, vec![falsified("P2", Measure::RepeatsOfOneKind, repeats)]));
    assert!(reply.text.contains(&format!("falsified at unix 2 ({repeats})")), "{}", reply.text);
    let (reply, added) = call(&mut ports, "status", "");
    assert_eq!(added, Vec::new(), "{}", reply.text);
    assert!(reply.text.contains(&format!("falsified at unix 2 ({repeats})")), "{}", reply.text);
    ports.log.records.extend(gaps(3).into_iter().map(|event| Record { at: Timestamp::from_unix_seconds(3), event }));
    let (_, added) = call(&mut ports, "status", "");
    assert_eq!(added, vec![falsified("P2", Measure::OpenEscapes, "open escapes 6")]);
}

#[test]
fn status_keeps_a_latched_falsification_when_resolving_escapes_makes_the_prediction_hold_again() {
    let at = |seconds: u64, event: Event| Record { at: Timestamp::from_unix_seconds(seconds), event };
    let mut ports = given([finished_task(), unclassified(6)].concat(), Vec::new());
    let (reply, added) = call(&mut ports, "status", "");
    let open = "open escapes 6";
    assert_eq!(added, vec![falsified("P2", Measure::OpenEscapes, open)], "{}", reply.text);
    ports.log.records.extend([1, 2, 3].map(|number| at(3, resolved(&format!("escape-{number}"), "task-1"))));
    let (reply, added) = call(&mut ports, "status", "");
    assert_eq!(added, Vec::new(), "{}", reply.text);
    let holds = "P2 Gaps get fixed, not bypassed: holds; observed open escapes 3, most repeats of one escape kind 0;";
    assert!(reply.text.contains(holds), "{}", reply.text);
    assert!(reply.text.contains(&format!("falsified at unix 2 ({open})")), "{}", reply.text);
}

#[test]
fn status_shows_a_latched_falsification_read_from_the_log_and_latches_nothing_for_a_prediction_that_holds() {
    let history = [finished_task(), vec![falsified("P6", Measure::MedianCiRuns, "median CI runs per merged PR 4")]];
    let status = status_of(history.concat());
    let p6 = status.lines().find(|line| line.contains("P6 Little rework")).unwrap_or_default();
    assert!(p6.contains("holds;") && p6.contains("falsified at unix 1 (median CI runs per merged PR 4)"), "{status}");
    let calm = status_of(finished_task());
    assert!(!calm.contains("falsified at"), "{calm}");
}

#[test]
fn status_shows_an_escape_of_a_kind_not_on_main_as_unchecked_and_counts_all_of_them_under_one_key() {
    let typos = [escaped("gpa", "one"), escaped("gpb", "two"), escaped("gap", "three"), escaped("gpc", "four")];
    let status = status_of(typos.to_vec());
    let marked = ["escape-1 [gpa, unchecked] one", "escape-2 [gpb, unchecked] two", "escape-4 [gpc, unchecked] four"];
    assert!(marked.iter().all(|line| status.contains(line)), "{status}");
    assert!(status.contains("escape-3 [gap] three"), "{status}");
    assert!(status.contains("P2 open escapes: 4 (0 closed as historical); most repeats of one kind: 2 ("), "{status}");
    assert!(
        status.contains("P2 Gaps get fixed, not bypassed: FALSIFIED; observed open escapes 4, most repeats"),
        "{status}"
    );
}

// Historical escapes (docs/stage1.md, "Historical escapes").

const ONE_OFF: &str = "bootstrap, meta did not exist yet";

#[test]
fn resolve_historical_closes_an_open_escape_with_no_task_and_it_no_longer_counts_as_open() {
    let mut ports = given(vec![escaped("gap", "no retire"), escaped("gap", "no review")], Vec::new());
    let (reply, events) = call(&mut ports, &format!("resolve escape-1 --historical {ONE_OFF}"), "");
    assert_eq!((reply.code, events), (Code::Done, vec![historical("escape-1", ONE_OFF)]));
    let (reply, _) = call(&mut ports, "status", "");
    assert!(reply.text.contains("open escapes: 1\n  escape-2 [gap] no review\n"), "{}", reply.text);
    assert!(reply.text.contains("P2 open escapes: 1 (1 closed as historical);"), "{}", reply.text);
    assert!(!reply.text.contains("escape-1"), "{}", reply.text);
    let (reply, _) = call(&mut ports, "escape gap again", "");
    assert!(reply.text.contains("open escapes: 2"), "{}", reply.text);
}

#[test]
fn status_p2_counts_escapes_closed_as_historical_including_corrections_but_not_task_resolutions() {
    let history = [
        finished_task(),
        vec![escaped("gap", "a"), escaped("gap", "b"), escaped("gap", "c"), escaped("gap", "d")],
        vec![historical("escape-1", "one-off"), resolved("escape-2", "task-1"), resolved("escape-3", "task-1")],
        vec![corrected("escape-3", "one-off")],
    ]
    .concat();
    let status = status_of(history);
    assert!(status.contains("P2 open escapes: 1 (2 closed as historical);"), "{status}");
}

#[test]
fn resolve_historical_is_refused_on_a_resolved_or_historical_or_unknown_escape_without_correcting() {
    let escapes = || vec![escaped("gap", "no retire")];
    let cases = [
        (
            [finished_task(), escapes(), vec![resolved("escape-1", "task-1")]].concat(),
            "already resolved against task-1",
        ),
        ([escapes(), vec![historical("escape-1", "first")]].concat(), "already historical: first"),
        (escapes(), "escape-2 is unknown"),
    ];
    cases.into_iter().for_each(|(history, why)| {
        let args = if why.contains("unknown") { "escape-2" } else { "escape-1" };
        let (reply, events) =
            call(&mut given(history, Vec::new()), &format!("resolve {args} --historical {ONE_OFF}"), "");
        assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
        assert!(reply.text.contains(why), "{}", reply.text);
    });
}

#[test]
fn resolve_historical_correcting_moves_a_task_resolution_to_historical_and_status_shows_both() {
    let history = [finished_task(), vec![escaped("gap", "no retire"), resolved("escape-1", "task-1")]].concat();
    let mut ports = given(history, Vec::new());
    let (reply, events) = call(&mut ports, &format!("resolve escape-1 --historical {ONE_OFF} --correcting"), "");
    assert_eq!((reply.code, events), (Code::Done, vec![corrected("escape-1", ONE_OFF)]));
    assert!(ports.log.records.iter().any(|record| record.event == resolved("escape-1", "task-1")));
    let (reply, _) = call(&mut ports, "status", "");
    let shown = format!("escape-1 resolved against task-1, corrected to historical: {ONE_OFF}");
    assert!(reply.text.contains(&shown) && reply.text.contains("open escapes: 0"), "{}", reply.text);
    assert!(reply.text.contains("P2 open escapes: 0 (1 closed as historical);"), "{}", reply.text);
    let (reply, events) = call(&mut ports, &format!("resolve escape-1 --historical {ONE_OFF} --correcting"), "");
    assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
    assert!(reply.text.contains("corrected to historical"), "{}", reply.text);
}

#[test]
fn resolve_historical_correcting_is_refused_on_an_open_or_historical_escape() {
    let cases = [
        (vec![escaped("gap", "no retire")], "is open, so there is nothing to correct"),
        (
            [vec![escaped("gap", "no retire")], vec![historical("escape-1", "first")]].concat(),
            "already historical: first",
        ),
    ];
    cases.into_iter().for_each(|(history, why)| {
        let (reply, events) =
            call(&mut given(history, Vec::new()), &format!("resolve escape-1 --historical {ONE_OFF} --correcting"), "");
        assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
        assert!(reply.text.contains(why), "{}", reply.text);
    });
}

#[test]
fn p1_still_counts_an_outside_merge_escape_that_names_a_pr_when_it_is_resolved_historical_or_corrected() {
    let outside =
        vec![opened("task-2"), claimed("task-2"), linked("task-2", "8"), merged_outside("8", 1), done("task-2")];
    let named = || vec![escaped_pr("outside-merge", "8", "merged by hand")];
    let closings = [
        vec![resolved("escape-1", "task-1")],
        vec![historical("escape-1", "one-off")],
        vec![resolved("escape-1", "task-1"), corrected("escape-1", "one-off")],
    ];
    closings.into_iter().for_each(|closing| {
        let history = [finished_task(), outside.clone(), named(), closing].concat();
        let mut ports = given(history, Vec::new());
        ports.github.listed = Some(Ok(listed(&[7, 8])));
        let (reply, _) = call(&mut ports, "status", "");
        assert!(reply.text.contains("P1 The loop is used: UNDECIDED; observed unrecorded bypasses unavailable (agent starts without a lease are not observable"), "{}", reply.text);
    });
}

#[test]
fn a_log_that_resolves_historically_twice_or_corrects_a_non_task_resolution_does_not_fold() {
    let one = || vec![escaped("gap", "no retire")];
    let bad = [
        [one(), vec![historical("escape-1", "a"), historical("escape-1", "b")]].concat(),
        [one(), vec![corrected("escape-1", "a")]].concat(),
        [one(), vec![historical("escape-1", "a"), corrected("escape-1", "b")]].concat(),
        [finished_task(), one(), vec![resolved("escape-1", "task-1"), historical("escape-1", "a")]].concat(),
    ];
    bad.into_iter().for_each(|history| {
        let (reply, _) = call(&mut given(history, Vec::new()), "status", "");
        assert_eq!(reply.code, Code::Failed, "{}", reply.text);
    });
}

// Lease deadlines (task-30, docs/stage1.md "Deadlines"). The fake clock stands at unix 2 and the
// given records at unix 1; the roles fixture gives work 45 min and review 15 min.

const FIXTURE_WORK_DEADLINE: u64 = 2 + 45 * 60;
const FIXTURE_REVIEW_DEADLINE: u64 = 2 + 15 * 60;

#[test]
fn claim_and_review_set_the_deadline_from_the_role_default_in_self_roles_and_the_brief_prints_it() {
    let (reply, events) = call(&mut given(vec![opened("task-1")], Vec::new()), "claim task-1 --agent alice", "");
    assert_eq!(events, vec![claimed("task-1"), leased_until("lease-1", "task-1", FIXTURE_WORK_DEADLINE)]);
    let line = format!("deadline: unix {FIXTURE_WORK_DEADLINE}, report or stop by then\n");
    assert!(reply.text.contains(&line), "{}", reply.text);
    let history = [linked_task(), vec![used("lease-1")]].concat();
    let (reply, events) = call(&mut given(history, vec![("7", pull(MOVED, &[]))]), "review task-1 --agent rev", "");
    assert_eq!(events, vec![review_leased_until("lease-2", MOVED, FIXTURE_REVIEW_DEADLINE)]);
    let line = format!("deadline: unix {FIXTURE_REVIEW_DEADLINE}, report or stop by then\n");
    assert!(reply.text.contains(&line), "{}", reply.text);
    let mut unreadable = given(vec![opened("task-1")], Vec::new());
    unreadable.git.files.remove(&SelfFile::Roles);
    let (reply, events) = call(&mut unreadable, "claim task-1 --agent alice", "");
    assert_eq!((reply.code, events), (Code::Failed, Vec::new()), "{}", reply.text);
    assert!(reply.text.contains("self/roles.toml"), "{}", reply.text);
}

#[test]
fn claim_and_review_take_a_deadline_override_in_minutes_and_self_claims_get_no_deadline() {
    let (reply, events) =
        call(&mut given(vec![opened("task-1")], Vec::new()), "claim task-1 --agent alice --deadline 10", "");
    assert_eq!(events, vec![claimed("task-1"), leased_until("lease-1", "task-1", 2 + 600)]);
    assert!(reply.text.contains("deadline: unix 602, report or stop by then"), "{}", reply.text);
    let history = [linked_task(), vec![used("lease-1")]].concat();
    let mut ports = given(history, vec![("7", pull(MOVED, &[]))]);
    let (_, events) = call(&mut ports, "review task-1 --agent rev --deadline 5", "");
    assert_eq!(events, vec![review_leased_until("lease-2", MOVED, 2 + 300)]);
    let (reply, events) = call(&mut given(vec![opened("task-1")], Vec::new()), "claim task-1 --self", "");
    assert_eq!((reply.code, events), (Code::Done, vec![self_claimed("task-1")]));
    assert!(reply.text.contains("deadline: none"), "{}", reply.text);
    let malformed = ["claim task-1 --self --deadline 10", "claim task-1 --agent a --deadline 0"];
    let more = ["claim task-1 --agent a --deadline 1.5", "review task-1 --agent a --deadline x"];
    malformed.into_iter().chain(more).for_each(|args| {
        let words: Vec<String> = args.split_whitespace().map(str::to_owned).collect();
        let reply = Call::parse(&words, String::new).unwrap_err();
        assert!(reply.code == Code::Failed && reply.text.contains("usage:"), "{args}: {}", reply.text);
    });
}

#[test]
fn the_started_record_carries_the_deadline_counted_from_the_start_time() {
    let history =
        [vec![opened("task-1"), claimed("task-1"), leased_until("lease-1", "task-1", 1 + 2700)], vec![used("lease-1")]]
            .concat();
    let mut ports = given(history, Vec::new());
    ports.clock.pause(100);
    let (reply, events) = call(&mut ports, "hook post-tool-use", &agent_ran("Agent", "lease-1", AGENT_ID));
    let deadline = Some(Timestamp::from_unix_seconds(102 + 2700));
    let started = Event::Started { lease: id("lease-1"), agent: id(AGENT_ID), deadline };
    assert_eq!((reply.text.as_str(), events), (ALLOW, vec![started]));
    ports.clock.pause(2750 - 102);
    let (status, _) = call(&mut ports, "status", "");
    assert!(
        status.text.contains("overdue leases (flagged)\n  (none)\n"),
        "past the lease's own deadline: {}",
        status.text
    );
    ports.clock.pause(2803 - 2750);
    let (status, _) = call(&mut ports, "status", "");
    let flagged = format!("lease-1 for task-1: agent {AGENT_ID}, deadline unix {}\n", 102 + 2700);
    assert!(status.text.contains(&flagged), "{}", status.text);
}

#[test]
fn status_flags_a_live_lease_past_its_deadline_with_its_agent_until_the_task_is_done() {
    let late = |deadline: u64| leased_until("lease-1", "task-1", deadline);
    let started = Event::Started { lease: id("lease-1"), agent: id(AGENT_ID), deadline: None };
    let running = vec![opened("task-1"), claimed("task-1"), late(1), used("lease-1"), started];
    let text = status_of(running.clone());
    let flagged = format!("overdue leases (flagged)\n  lease-1 for task-1: agent {AGENT_ID}, deadline unix 1\n");
    assert!(text.contains(&flagged), "{text}");
    let text = status_of(vec![opened("task-1"), claimed("task-1"), late(1)]);
    assert!(text.contains("overdue leases (flagged)\n  lease-1 for task-1: unused, deadline unix 1\n"), "{text}");
    let unbound = vec![opened("task-1"), claimed("task-1"), late(1), used("lease-1")];
    assert!(status_of(unbound).contains("lease-1 for task-1: agent unknown, deadline unix 1\n"));
    let none = "overdue leases (flagged)\n  (none)\n";
    let on_time = status_of(vec![opened("task-1"), claimed("task-1"), late(2)]);
    assert!(on_time.contains(none), "{on_time}");
    let finished = [running, vec![linked("task-1", "7"), merged("7", 1), done("task-1")]].concat();
    assert!(status_of(finished).contains(none));
    let reviewed = [
        linked_task(),
        vec![review_leased_until("lease-2", PINNED, 1), used("lease-2")],
        vec![Event::Reviewed { lease: id("lease-2"), verdict: id("pass") }],
    ]
    .concat();
    assert!(status_of(reviewed).contains(none));
    let resumes = status_of(bound());
    let order = (resumes.find("leases resumed 3 or more times"), resumes.find("overdue leases (flagged)"));
    assert!(matches!(order, (Some(resumed), Some(overdue)) if overdue > resumed), "{resumes}");
}

#[test]
fn leases_from_before_deadlines_fold_as_no_deadline_and_are_never_overdue() {
    let text = status_of(bound());
    assert!(text.contains("overdue leases (flagged)\n  (none)\n"), "{text}");
    let text = status_of(claimed_task());
    assert!(text.contains("lease-1 for task-1, no deadline"), "{text}");
    let old = r#"{"at":1,"event":"Leased","lease":1,"task":1}"#;
    assert_eq!(serde_json::from_str::<Record>(old).unwrap().event, leased("lease-1", "task-1"));
    let old = r#"{"at":1,"event":"Started","lease":1,"agent":"a4d2c8f1e0b3a297"}"#;
    assert_eq!(serde_json::from_str::<Record>(old).unwrap().event, started("lease-1", AGENT_ID));
    let new = Record { at: Timestamp::from_unix_seconds(1), event: leased_until("lease-1", "task-1", 9) };
    let line = serde_json::to_string(&new).unwrap();
    assert_eq!(line, r#"{"at":1,"event":"Leased","lease":1,"task":1,"deadline":9}"#);
}

// Task 31 (critic issue #11): P1 judges agent starts too, and a hook denial is not a miss.

#[test]
fn status_never_shows_p1_as_holds_because_meta_cannot_observe_starts_without_a_lease() {
    let denied = vec![Event::Denied { attempt: Attempt::AgentStart }, Event::Denied { attempt: Attempt::AgentStart }];
    let status = status_of([finished_task(), denied].concat());
    assert!(status.contains("P1 The loop is used: UNDECIDED; observed unrecorded bypasses unavailable (agent starts without a lease are not observable"), "{status}");
    assert!(
        !status.contains("P1 The loop is used: holds") && !status.contains("P1 The loop is used: FALSIFIED"),
        "{status}"
    );
    assert!(status.contains("prevented by the hook: 2;"), "{status}");
}

#[test]
fn status_falsifies_p1_for_each_outside_start_escape_and_hook_denials_never_count() {
    let start = |reason: &str| escaped("outside-start", reason);
    let cases = [
        (vec![start("agent without a lease")], "FALSIFIED; observed unrecorded bypasses 1;"),
        (vec![start("one"), start("two")], "FALSIFIED; observed unrecorded bypasses 2;"),
        (
            vec![start("one"), Event::Denied { attempt: Attempt::AgentStart }],
            "FALSIFIED; observed unrecorded bypasses 1;",
        ),
    ];
    cases.into_iter().for_each(|(escapes, expected)| {
        let status = status_of([finished_task(), escapes].concat());
        assert!(status.contains(expected), "{expected}\n{status}");
    });
    let mut ports = given([finished_task(), vec![start("no lease")]].concat(), Vec::new());
    ports.github.listed = Some(Err(PortError::new("gh is down")));
    let (reply, _) = call(&mut ports, "status", "");
    assert!(reply.text.contains("P1 The loop is used: FALSIFIED; observed unrecorded bypasses 1;"), "{}", reply.text);
    assert!(reply.text.contains("bypassed, from outside-start escapes: 1;"), "{}", reply.text);
}
