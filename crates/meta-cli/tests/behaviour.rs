use meta_cli::{Answer, Call, Code, answer};
use meta_core::{
    Attempt, Build, Checked, Claimant, Event, Measure, Outcome, Place, PrNumber, Record, Route, Rules, RunCount,
    SelfFile, Sha, SideLine, Span, Standing, Timestamp, UnverifiedEscape, UnverifiedEscapes,
};
use meta_ports::memory::{FakeClock, MemoryLog, RecordedGit, RecordedGitHub};
use meta_ports::real::JsonlLog;
use meta_ports::{Clock, GhReplies, Log, PortError, Ports};
use meta_testkit::Scratch;
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
const MAIN_BUILD: &str = "c0ffee0";
const BRANCH_BUILD: &str = "b4a2c40";

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
    Event::Leased { lease: id(lease), task: id(task), agent: None, deadline: None }
}

/// A lease from this change on: its deadline in unix seconds.
fn leased_until(lease: &str, task: &str, deadline: u64) -> Event {
    leased_to("alice", lease, task, deadline)
}

/// A lease from this change on, with the agent it is for.
fn leased_to(agent: &str, lease: &str, task: &str, deadline: u64) -> Event {
    Event::Leased {
        lease: id(lease),
        task: id(task),
        agent: Some(id(agent)),
        deadline: Some(Timestamp::from_unix_seconds(deadline)),
    }
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
    Event::Escaped { kind: id(kind), reason: id(reason), pr: None, checked: Checked::Listed, imported: None }
}

fn escaped_pr(kind: &str, pr: &str, reason: &str) -> Event {
    Event::Escaped { kind: id(kind), reason: id(reason), pr: Some(id(pr)), checked: Checked::Listed, imported: None }
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
    merge_state: &'a str,
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
            "mergeStateStatus": self.merge_state,
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
        merge_state: "CLEAN",
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
        log: MemoryLog {
            records: events.into_iter().map(|event| Record { at, event }).collect(),
            broken: None,
            place: Place::Elsewhere,
            set_aside: Vec::new(),
        },
        git: RecordedGit {
            origin: id(ORIGIN),
            files: self_files(),
            local: self_files(),
            branches: BTreeMap::from([(id("task-1"), id(PINNED))]),
            on_main: [(id(MAIN_BUILD), true), (id(BRANCH_BUILD), false)].into(),
        },
        github: RecordedGitHub {
            earlier: RefCell::default(),
            replies: pulls.into_iter().map(|(pr, replies)| (id::<PrNumber>(pr), replies)).collect(),
            merged: Vec::new(),
            updated: Vec::new(),
            listed: Some(Ok("[]".to_owned())),
        },
        clock: FakeClock::at(Timestamp::from_unix_seconds(2)),
        build: Build::commit(id(MAIN_BUILD)),
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

/// The words of a command line, as `Call::parse` takes them.
fn command(args: &[&str]) -> Vec<String> {
    args.iter().map(|word| (*word).to_owned()).collect()
}

fn call(ports: &mut Memory, args: &[&str], stdin: &str) -> (Answer, Vec<Event>) {
    let before = ports.log.records.len();
    let reply = answer(Call::parse(&command(args), || stdin.to_owned()).unwrap(), ports);
    let added = ports.log.records[before..].iter().map(|record| record.event.clone()).collect();
    (reply, added)
}

fn parsed(args: &[&str]) -> Call {
    Call::parse(&command(args), String::new).unwrap()
}

fn hook(events: Vec<Event>, input: &str) -> (Answer, Vec<Event>) {
    call(&mut given(events, Vec::new()), &["hook", "pre-tool-use"], input)
}

fn agent(description: &str) -> String {
    serde_json::json!({ "tool_name": "Agent", "tool_input": { "description": description, "prompt": "Work on task-1." } })
        .to_string()
}

#[test]
fn open_prints_a_new_task_id_and_records_opened() {
    let (reply, events) = call(&mut given(vec![opened("task-1")], Vec::new()), &["open", "Fix", "the", "hook"], "");
    assert_eq!(reply, Answer { text: "task-2\n".to_owned(), code: Code::Done });
    assert_eq!(events, vec![opened("task-2")]);
}

#[test]
fn claim_of_an_open_unclaimed_task_leases_and_prints_the_brief_with_every_sourced_rule() {
    let (reply, events) =
        call(&mut given(vec![opened("task-1")], Vec::new()), &["claim", "task-1", "--agent", "alice"], "");
    assert_eq!(events, vec![claimed("task-1"), leased_until("lease-1", "task-1", FIXTURE_WORK_DEADLINE)]);
    assert_eq!(reply.code, Code::Done);
    let rules = Rules::parse(RULES).unwrap().to_string();
    assert!(reply.text.contains(&rules), "{}", reply.text);
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
        let (reply, added) = call(&mut given(events, Vec::new()), &["claim", task, "--agent", "bob"], "");
        assert_eq!((reply.code, added), (Code::Refused, Vec::new()));
        assert!(reply.text.contains(reason), "{}", reply.text);
    });
    let (reply, _) = call(&mut given(claimed_task(), Vec::new()), &["claim", "task-1", "--agent", "bob"], "");
    assert!(
        reply.text.contains("once it is spent, meta claim task-1 --agent <name> issues a fresh lease"),
        "{}",
        reply.text
    );
    assert!(reply.text.contains("meta brief lease-1 prints its brief again"), "{}", reply.text);
}

#[test]
fn claim_of_a_claimed_task_whose_lease_is_spent_issues_a_fresh_lease() {
    let spent = Event::LeaseUsed { lease: id("lease-1") };
    [claimed_task(), linked_task()].into_iter().for_each(|history| {
        let history = [history, vec![spent.clone()]].concat();
        let (reply, events) = call(&mut given(history, Vec::new()), &["claim", "task-1", "--agent", "bob"], "");
        let fresh = leased_to("bob", "lease-2", "task-1", FIXTURE_WORK_DEADLINE);
        assert_eq!((reply.code, events), (Code::Done, vec![fresh]));
        assert!(reply.text.contains("lease: lease-2") && reply.text.contains("rules:"), "{}", reply.text);
    });
}

#[test]
fn link_of_a_claimed_task_records_linked_with_the_head_pinned() {
    let mut ports = given(claimed_task(), vec![("7", pull(PINNED, &[]))]);
    let (reply, events) = call(&mut ports, &["link", "task-1", "7"], "");
    assert_eq!(reply.code, Code::Done);
    assert_eq!(events, vec![linked("task-1", "7")]);
}

#[test]
fn merge_of_a_linked_green_unmoved_pr_squash_merges_with_the_head_pinned() {
    let mut ports = given(linked_task(), vec![("7", pull(PINNED, &[Outcome::Green, Outcome::Green]))]);
    let (reply, events) = call(&mut ports, &["merge", "7"], "");
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
        let (reply, events) = call(&mut ports, &["merge", pr], "");
        assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
        assert!(reply.text.contains(which) && reply.text.contains(next), "{}", reply.text);
        assert!(ports.github.merged.is_empty());
    });
}

#[test]
fn merge_of_a_pr_behind_main_is_refused_with_the_update_step_and_does_not_merge() {
    let behind = Gh { merge_state: "BEHIND", ..gh(PINNED, &[Outcome::Green]) }.replies();
    let mut ports = given(linked_task(), vec![("7", behind)]);
    let (reply, events) = call(&mut ports, &["merge", "7"], "");
    assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
    for why in ["behind main", "meta merge 7 --update", "wait for CI", "meta link task-1 7", "meta merge 7 again"] {
        assert!(reply.text.contains(why), "{}", reply.text);
    }
    assert!(ports.github.merged.is_empty() && ports.github.updated.is_empty());
}

#[test]
fn merge_update_of_a_pr_behind_main_updates_the_branch_and_does_not_merge_in_the_same_call() {
    let behind = Gh { merge_state: "BEHIND", ..gh(PINNED, &[Outcome::Green]) }.replies();
    let mut ports = given(linked_task(), vec![("7", behind)]);
    let (reply, events) = call(&mut ports, &["merge", "7", "--update"], "");
    assert_eq!((reply.code, events), (Code::Done, Vec::new()));
    assert!(reply.text.contains("updated; CI must pass on the new head before merging"), "{}", reply.text);
    assert_eq!(ports.github.updated, vec![id::<PrNumber>("7")]);
    assert!(ports.github.merged.is_empty());
}

#[test]
fn merge_update_is_refused_for_a_pr_that_is_not_behind_is_unlinked_or_is_not_open() {
    let cases = [
        ("7", pull(PINNED, &[Outcome::Green]), "not behind main"),
        ("8", pull(PINNED, &[Outcome::Green]), "#8 is unlinked"),
        ("7", Gh { state: "CLOSED", merge_state: "BEHIND", ..gh(PINNED, &[]) }.replies(), "not open"),
    ];
    cases.into_iter().for_each(|(pr, recorded, why)| {
        let mut ports = given(linked_task(), vec![(pr, recorded)]);
        let (reply, events) = call(&mut ports, &["merge", pr, "--update"], "");
        assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
        assert!(reply.text.contains(why), "{}", reply.text);
        assert!(ports.github.updated.is_empty() && ports.github.merged.is_empty());
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
    let pulls = vec![("9", ran(1)), ("10", ran(1))];
    let (reply, events) = call(&mut given(history, pulls), &["status"], "");
    assert_eq!(events, vec![falsified("P1", Measure::UnrecordedBypasses, "unrecorded bypasses 1")]);
    let expected = "open tasks
  task-1 claimed by alice: Fix the hook
  task-3 open: Fix the hook
deferred
  (none)
live leases
  lease-1 for task-1, no deadline
open escapes: 3
  escape-1 [gap] gh is down
  escape-2 [gap] gh is down again
  escape-3 [outside-merge] hook too strict
escapes corrected to historical
  (none)
latest review verdict per linked PR
  (none)
leases resumed 3 or more times (flagged)
  (none)
overdue leases (flagged)
  (none)
waiting on the owner (flagged)
  (none)
idle (flagged)
  idle: no live work; stalled: task-1 (claimed), task-3 (open)
P1 agent starts through meta: 1; prevented by the hook: 1; bypassed, from outside-start escapes: 0; self-claims, not agent starts: 0
P1 merges through meta: 1 of 2
P1 merged outside meta: 1 (0 missing from the log)
P2 open escapes: 3 (0 closed as historical); most repeats of one kind: 1 (gap)
P6 CI runs per merged PR: 6 runs over 2 merged PRs; median 3
rework per task (report only; not part of P6)
  task-2 (#9, done): failed review rounds 0, CI runs 4
  task-4 (#10, done): failed review rounds 0, CI runs 2
rework total: tasks with 2 or more failed review rounds: 0 of 2 tasks with a PR; worst: task-2 (failed review rounds 0, CI runs 4)
predictions
";
    assert_eq!(reply.text.get(..expected.len()), Some(expected), "{}", reply.text);
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
    assert_eq!(call(&mut ports, &["merge", "7"], "").1, vec![merged("7", 3), done("task-1")]);
    let red = Gh { rollup: vec![context("ERROR"), lint], ..gh(PINNED, &[]) }.replies();
    let (reply, events) = call(&mut given(linked_task(), vec![("7", red)]), &["merge", "7"], "");
    assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
    assert!(reply.text.contains("red check gates") && !reply.text.contains("lint"), "{}", reply.text);
}

#[test]
fn merge_reports_a_failed_protection_lookup_or_unreadable_gh_json_as_a_port_failure() {
    let failed = Gh { required: Err("gh api: HTTP 404"), ..gh(PINNED, &[Outcome::Green]) }.replies();
    let garbled = GhReplies { view: "{".to_owned(), ..pull(PINNED, &[Outcome::Green]) };
    [
        (failed, "error: gh api: HTTP 404\n"),
        (garbled, "error: gh pr view: EOF while parsing an object at line 1 column 1\n"),
    ]
    .into_iter()
    .for_each(|(replies, why)| {
        let mut ports = given(linked_task(), vec![("7", replies)]);
        let (reply, events) = call(&mut ports, &["merge", "7"], "");
        assert_eq!((reply.code, events), (Code::Failed, Vec::new()));
        assert_eq!(reply.text, why);
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
fn link_waits_while_github_still_reports_the_previous_head_then_pins_the_pushed_commit() {
    let mut ports = given(claimed_task(), vec![("7", pull(PINNED, &[]))]);
    ports.git.branches.insert(id("task-1"), id(MOVED));
    let lagging = VecDeque::from([pull(PINNED, &[]), pull(PINNED, &[]), pull(MOVED, &[])]);
    ports.github.earlier.borrow_mut().insert(id("7"), lagging);
    let start = ports.clock.now();
    let (reply, events) = call(&mut ports, &["link", "task-1", "7"], "");
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
    let (reply, events) = call(&mut ports, &["link", "task-1", "7"], "");
    assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
    for why in [PINNED, MOVED, "push has not landed", "git push origin task-1"] {
        assert!(reply.text.contains(why), "{}", reply.text);
    }
    assert_eq!(ports.clock.now(), start.advanced(60));
}

#[test]
fn link_retries_a_transient_github_error_until_it_reports_the_pushed_head() {
    let mut ports = given(claimed_task(), vec![("7", pull(MOVED, &[]))]);
    ports.git.branches.insert(id("task-1"), id(MOVED));
    let flaky = GhReplies { required: Err(PortError::new("gh api: HTTP 502")), ..pull(PINNED, &[]) };
    ports.github.earlier.borrow_mut().insert(id("7"), VecDeque::from([pull(PINNED, &[]), flaky.clone(), flaky]));
    let start = ports.clock.now();
    let (reply, events) = call(&mut ports, &["link", "task-1", "7"], "");
    assert_eq!(reply.code, Code::Done, "{}", reply.text);
    assert_eq!(events, vec![Event::Linked { task: id("task-1"), pr: id("7"), pin: id(MOVED) }]);
    assert!(ports.clock.now() > start && ports.clock.now() < start.advanced(60));
}

#[test]
fn link_fails_with_the_github_error_when_it_lasts_until_the_deadline() {
    let down = GhReplies { required: Err(PortError::new("gh api: HTTP 502")), ..pull(PINNED, &[]) };
    let mut ports = given(claimed_task(), vec![("7", down)]);
    let start = ports.clock.now();
    let (reply, events) = call(&mut ports, &["link", "task-1", "7"], "");
    assert_eq!((reply.code, events), (Code::Failed, Vec::new()));
    assert_eq!(reply.text, "error: gh api: HTTP 502\n");
    assert_eq!(ports.clock.now(), start.advanced(60));
}

#[test]
fn link_refuses_with_the_next_step_when_there_is_no_local_branch_for_the_task() {
    let mut ports = given(claimed_task(), vec![("7", pull(PINNED, &[]))]);
    ports.git.branches.clear();
    let start = ports.clock.now();
    let (reply, events) = call(&mut ports, &["link", "task-1", "7"], "");
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
        let (reply, added) = call(&mut given(events, vec![("7", replies.replies())]), &["link", "task-1", "7"], "");
        assert_eq!((reply.code, added), (Code::Refused, Vec::new()));
        assert!(reply.text.contains(why), "{}", reply.text);
    });
}

#[test]
fn parallel_agent_starts_through_the_jsonl_log_spend_one_lease_once() {
    let scratch = Scratch::new("lease-race");
    let path = scratch.join("log.jsonl");
    let at = Timestamp::from_unix_seconds(1);
    let seed: Vec<Record> = claimed_task().into_iter().map(|event| Record { at, event }).collect();
    JsonlLog::at(path.clone()).transact(|_, _| (seed, ())).unwrap();
    let start = Call::Hook(meta_core::Phase::Before, agent("lease-1"));
    let replies: Vec<Answer> = std::thread::scope(|scope| {
        let racers: Vec<_> = (0..8)
            .map(|_| {
                scope.spawn(|| {
                    let Ports { git, github, clock, build, .. } = given(Vec::new(), Vec::new());
                    let mut ports = Ports { log: JsonlLog::at(path.clone()), git, github, clock, build };
                    answer(start.clone(), &mut ports)
                })
            })
            .collect();
        racers.into_iter().map(|racer| racer.join().unwrap()).collect()
    });
    assert_eq!(replies.iter().filter(|reply| reply.text == ALLOW).count(), 1);
}

#[test]
fn unknown_or_malformed_commands_answer_with_usage() {
    [
        &[][..],
        &["frobnicate"][..],
        &["claim", "task-1"][..],
        &["claim", "one", "--agent", "alice"][..],
        &["merge", "seven"][..],
        &["open"][..],
        &["hook", "stop"][..],
        &["escape", "gap"][..],
        &["escape", "gap", "--pr", "seven", "merged"][..],
        &["resolve", "escape-1"][..],
        &["resolve", "escape-1", "--historical"][..],
        &["resolve", "escape-1", "--historical", "--correcting"][..],
        &["verdict", "task-1", "maybe"][..],
        &["retire", "task-1"][..],
        &["claim", "task-1", "--self", "bob"][..],
    ]
    .into_iter()
    .for_each(|args| {
        let refused = Call::parse(&command(args), String::new).unwrap_err();
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
        assert_eq!(call(&mut corrupt(), &["hook", "pre-tool-use"], &bash(command)), (allowed.clone(), Vec::new()));
        assert_eq!(call(&mut unopenable(), &["hook", "pre-tool-use"], &bash(command)).0, allowed, "{command}");
    });
}

#[test]
fn a_corrupt_or_unopenable_log_still_records_escapes_but_denies_agent_starts() {
    let corrupt = || vec![claimed("task-1")];
    let (reply, events) = call(&mut given(corrupt(), Vec::new()), &["escape", "gap", "log", "is", "corrupt"], "");
    assert_eq!((reply.code, events), (Code::Done, vec![escaped("gap", "log is corrupt")]));
    let repair = "Next: meta escape <kind> <reason>, then repair memory log";
    let (reply, events) = hook(corrupt(), &agent("lease-1"));
    assert_eq!(events, Vec::new());
    assert!(reply.text.contains(DENY) && reply.text.contains("fail closed"), "{}", reply.text);
    assert!(reply.text.contains(repair), "{}", reply.text);
    let (reply, events) = call(&mut given(corrupt(), Vec::new()), &["status"], "");
    assert_eq!((reply.code, events), (Code::Failed, Vec::new()));
    assert!(reply.text.contains("memory log record 1") && reply.text.contains(repair), "{}", reply.text);
    let mut unopenable = given(claimed_task(), Vec::new());
    unopenable.log.broken = Some(PortError::new("cannot open the log"));
    let (reply, _) = call(&mut unopenable, &["hook", "pre-tool-use"], &agent("lease-1"));
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
    let (reply, events) = call(&mut ports, &["status"], "");
    assert_eq!(events, Vec::new());
    let expected = ["P1 merges through meta: 1 of 1", "P1 merged outside meta: 0 (0 missing from the log)"];
    assert!(expected.iter().all(|line| reply.text.contains(line)), "{}", reply.text);
}

#[test]
fn status_counts_a_pr_merged_on_github_but_missing_from_the_log_as_merged_outside_meta() {
    let history = [linked_task(), vec![merged("7", 1), done("task-1")]].concat();
    let mut ports = given(history, Vec::new());
    ports.github.listed = Some(Ok(listed(&[7, 12, 13])));
    let (reply, events) = call(&mut ports, &["status"], "");
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
        let (reply, events) = call(&mut ports, &["status"], "");
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
    let (reply, events) = call(&mut ports, &["status"], "");
    assert_eq!((reply.code, events), (Code::Done, Vec::new()));
    assert!(reply.text.contains("P1 merges: unavailable") && reply.text.contains("truncated"), "{}", reply.text);
    let mut ports = given(claimed_task(), Vec::new());
    ports.github.listed = Some(Ok(full(1000)));
    let (reply, _) = call(&mut ports, &["status"], "");
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
    Event::ReviewLeased {
        lease: id(lease),
        task: id("task-1"),
        agent: None,
        pr: id("7"),
        head: id(head),
        deadline: None,
    }
}

fn review_leased_until(lease: &str, head: &str, deadline: u64) -> Event {
    let deadline = Some(Timestamp::from_unix_seconds(deadline));
    Event::ReviewLeased {
        lease: id(lease),
        task: id("task-1"),
        agent: Some(id("rev")),
        pr: id("7"),
        head: id(head),
        deadline,
    }
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
    call(&mut given(events, Vec::new()), &["hook", "post-tool-use"], input)
}

fn bound() -> Vec<Event> {
    [claimed_task(), vec![used("lease-1"), started("lease-1", AGENT_ID)]].concat()
}

fn status_of(events: Vec<Event>) -> String {
    let (reply, added) = call(&mut given(events, Vec::new()), &["status"], "");
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
    let (reply, events) = call(&mut given(history, Vec::new()), &["escape", "gap", "no", "self-claim"], "");
    assert_eq!((reply.code, events), (Code::Done, vec![escaped("gap", "no self-claim")]));
    let expected = ["escape-4 recorded: [gap] no self-claim", "open escapes: 3", "repeats of kind gap: 2"];
    assert!(expected.iter().all(|line| reply.text.contains(line)), "{}", reply.text);
}

#[test]
fn escape_of_an_unknown_kind_is_refused_and_lists_the_kinds() {
    [&["escape", "hook", "too", "strict"][..], &["escape", "unclassified", "old", "text"]].into_iter().for_each(
        |args| {
            let (reply, events) = call(&mut given(Vec::new(), Vec::new()), args, "");
            assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
            let listed =
                ["is not an escape kind", "- gap: real work needed", "source: fixture: gap", "- outside-merge:"];
            assert!(listed.iter().all(|part| reply.text.contains(part)), "{}", reply.text);
        },
    );
}

#[test]
fn resolve_against_a_done_task_records_resolved_and_the_escape_no_longer_counts_as_open() {
    let history = [finished_task(), vec![escaped("gap", "no retire"), escaped("gap", "no review")]].concat();
    let mut ports = given(history, Vec::new());
    let (reply, events) = call(&mut ports, &["resolve", "escape-1", "task-1"], "");
    assert_eq!((reply.code, events), (Code::Done, vec![resolved("escape-1", "task-1")]));
    let (reply, _) = call(&mut ports, &["status"], "");
    assert!(reply.text.contains("open escapes: 1\n  escape-2 [gap] no review\n"), "{}", reply.text);
    assert!(reply.text.contains("P2 open escapes: 1 (0 closed as historical);"), "{}", reply.text);
    assert!(!reply.text.contains("escape-1"), "{}", reply.text);
}

#[test]
fn resolve_is_refused_with_the_reason_when_the_task_is_not_done_or_the_escape_is_resolved_or_unknown() {
    let escapes = || vec![escaped("gap", "no retire")];
    let cases = [
        ([claimed_task(), escapes()].concat(), ["escape-1", "task-1"], "only against a done task"),
        (
            [finished_task(), escapes(), vec![resolved("escape-1", "task-1")]].concat(),
            ["escape-1", "task-1"],
            "already resolved",
        ),
        ([finished_task(), escapes()].concat(), ["escape-2", "task-1"], "escape-2 is unknown"),
        ([finished_task(), escapes()].concat(), ["escape-0", "task-1"], "escape-0 is unknown"),
        ([finished_task(), escapes()].concat(), ["escape-1", "task-9"], "task-9 is unknown"),
    ];
    cases.into_iter().for_each(|(history, args, why)| {
        let (reply, events) = call(&mut given(history, Vec::new()), &["resolve", args[0], args[1]], "");
        assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
        assert!(reply.text.contains(why), "{}", reply.text);
    });
}

#[test]
fn escapes_logged_before_kinds_fold_as_unclassified_beside_old_denied_and_lease_used_records() {
    let text = include_str!("fixtures/stage0-log.jsonl");
    let records: Vec<Record> = serde_json::Deserializer::from_str(text).into_iter().map(Result::unwrap).collect();
    let unclassified = |reason: &str| escaped("unclassified", reason);
    assert_eq!(records[2].event, unclassified("bootstrap: agent started without a lease (meta does not exist yet)"));
    let mut ports = given(Vec::new(), Vec::new());
    ports.log.records = records;
    let (reply, _) = call(&mut ports, &["status"], "");
    let expected = [
        "escape-1 [unclassified] bootstrap: agent started",
        "escape-2 [unclassified] no retire command",
        "P1 agent starts through meta: 1; prevented by the hook: 1; bypassed, from outside-start escapes: 0;",
        "P2 open escapes: 2 (0 closed as historical); most repeats of one kind: 0",
    ];
    assert!(expected.iter().all(|line| reply.text.contains(line)), "{}", reply.text);
    let (reply, events) = call(&mut ports, &["resolve", "escape-1", "task-1"], "");
    assert_eq!((reply.code, events), (Code::Done, vec![resolved("escape-1", "task-1")]));
}

#[test]
fn claim_self_of_an_open_task_records_claimed_by_the_lead_with_no_lease_and_prints_the_brief() {
    let (reply, events) = call(&mut given(vec![opened("task-1")], Vec::new()), &["claim", "task-1", "--self"], "");
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
    let (reply, events) = call(&mut given(claimed_task(), Vec::new()), &["claim", "task-1", "--self"], "");
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
        let (reply, events) = call(&mut ports, &["retire", "task-1", "moot", "after", "#5"], "");
        let retired = Event::Retired { task: id("task-1"), reason: id("moot after #5") };
        assert_eq!((reply.code, events), (Code::Done, vec![retired]), "{}", reply.text);
        let (status, _) = call(&mut ports, &["status"], "");
        assert!(status.text.contains("open tasks\n  (none)\n"), "{}", status.text);
        assert!(status.text.contains("live leases\n  (none)\n"), "{}", status.text);
    });
    let open = given(linked_task(), vec![("7", pull(PINNED, &[]))]);
    let (reply, events) = call(&mut { open }, &["retire", "task-1", "moot"], "");
    assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
    assert!(reply.text.contains("#7 is linked and open"), "{}", reply.text);
}

#[test]
fn defer_of_an_open_task_records_deferred_and_lists_it_under_deferred() {
    let mut ports = given(vec![opened("task-1")], Vec::new());
    let (reply, events) = call(&mut ports, &["defer", "task-1", "after", "stage", "2"], "");
    let deferred = Event::Deferred { task: id("task-1"), reason: id("after stage 2") };
    assert_eq!((reply.code, events), (Code::Done, vec![deferred]));
    let (status, _) = call(&mut ports, &["status"], "");
    assert!(
        status.text.contains("open tasks\n  (none)\ndeferred\n  task-1 deferred (after stage 2)"),
        "{}",
        status.text
    );
    let (reply, events) = call(&mut given(claimed_task(), Vec::new()), &["defer", "task-1", "later"], "");
    assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
}

#[test]
fn undefer_of_a_deferred_task_records_undeferred_and_opens_it_again() {
    let history = vec![opened("task-1"), Event::Deferred { task: id("task-1"), reason: id("later") }];
    let mut ports = given(history, Vec::new());
    let (reply, events) = call(&mut ports, &["undefer", "task-1"], "");
    assert_eq!((reply.code, events), (Code::Done, vec![Event::Undeferred { task: id("task-1") }]));
    let (status, _) = call(&mut ports, &["status"], "");
    assert!(status.text.contains("open tasks\n  task-1 open") && status.text.contains("deferred\n  (none)"));
    let (reply, events) = call(&mut given(vec![opened("task-1")], Vec::new()), &["undefer", "task-1"], "");
    assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
}

#[test]
fn review_of_a_task_linked_to_an_open_pr_leases_and_prints_an_unprimed_brief_with_the_review_rule() {
    let history = [linked_task(), vec![used("lease-1")]].concat();
    let mut ports = given(history, vec![("7", pull(MOVED, &[]))]);
    let (reply, events) = call(&mut ports, &["review", "task-1", "--agent", "rev"], "");
    let expected = vec![review_leased_until("lease-2", MOVED, FIXTURE_REVIEW_DEADLINE)];
    assert_eq!((reply.code, events), (Code::Done, expected), "{}", reply.text);
    let brief = ["review: #7 for task-1", "agent: rev", "lease: lease-2", "goal:", "constraints:", "review-gates"];
    assert!(brief.iter().all(|line| reply.text.contains(line)), "{}", reply.text);
    let (status, _) = call(&mut ports, &["status"], "");
    assert!(status.text.contains("lease-2 for task-1"), "{}", status.text);
    let refusals = [
        (claimed_task(), pull(PINNED, &[]), "only a task linked to an open PR"),
        (linked_task(), Gh { state: "CLOSED", ..gh(PINNED, &[]) }.replies(), "#7 is not open"),
    ];
    refusals.into_iter().for_each(|(history, replies, why)| {
        let (reply, events) =
            call(&mut given(history, vec![("7", replies)]), &["review", "task-1", "--agent", "rev"], "");
        assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
        assert!(reply.text.contains(why), "{}", reply.text);
    });
}

#[test]
fn verdict_after_a_used_review_lease_records_reviewed_and_status_shows_the_latest_verdict_per_linked_pr() {
    let reviewed = |verdict: &str| Event::Reviewed { lease: id("lease-2"), verdict: id(verdict) };
    let (reply, events) = call(&mut given(linked_task(), Vec::new()), &["verdict", "task-1", "pass"], "");
    assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
    let unused = [linked_task(), vec![review_leased("lease-2", PINNED)]].concat();
    let (reply, _) = call(&mut given(unused.clone(), Vec::new()), &["verdict", "task-1", "pass"], "");
    assert!(reply.code == Code::Refused && reply.text.contains("awaits a verdict"), "{}", reply.text);
    let mut ports = given([unused, vec![used("lease-2")]].concat(), Vec::new());
    let (reply, events) = call(&mut ports, &["verdict", "task-1", "fail"], "");
    assert_eq!((reply.code, events), (Code::Done, vec![reviewed("fail")]));
    let (reply, _) = call(&mut ports, &["verdict", "task-1", "pass"], "");
    assert_eq!(reply.code, Code::Refused);
    let later = vec![
        review_leased("lease-3", MOVED),
        used("lease-3"),
        Event::Reviewed { lease: id("lease-3"), verdict: id("pass") },
    ];
    ports.log.records.extend(later.into_iter().map(|event| Record { at: Timestamp::from_unix_seconds(3), event }));
    let (status, _) = call(&mut ports, &["status"], "");
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
        "P1 The loop is used: UNDECIDED; observed unrecorded bypasses unavailable (agent starts without a lease are not observable",
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
    let (reply, _) = call(&mut ports, &["status"], "");
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
    let (reply, _) = call(&mut ports, &["status"], "");
    assert!(
        reply.text.contains(
            "P1 The loop is used: UNDECIDED; observed unrecorded bypasses unavailable (agent starts without a lease are not observable"
        ),
        "{}",
        reply.text
    );
    ports.github.listed = Some(Err(PortError::new("gh is down")));
    let (reply, _) = call(&mut ports, &["status"], "");
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
        call(&mut ports, &["status"], "").0.text
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
    let (reply, events) = call(&mut ports, &["escape", "outside-merge", "--pr", "12", "merged", "by", "hand"], "");
    assert_eq!((reply.code, events), (Code::Done, vec![escaped_pr("outside-merge", "12", "merged by hand")]));
    assert!(reply.text.contains("escape-1 recorded: [outside-merge] #12 merged by hand"), "{}", reply.text);
    let refusals = [
        (&["escape", "outside-merge", "merged", "by", "hand"][..], "outside-merge names the PR merged outside meta"),
        (&["escape", "gap", "--pr", "12", "no", "retire"], "gap names no PR"),
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
    let (reply, events) = call(&mut ports, &["escape", "gap", "no", "retire"], "");
    assert_eq!((reply.code, events), (Code::Done, vec![escaped("gap", "no retire")]), "{}", reply.text);
    ports.git.local.clear();
    let unchecked = |kind: &str, reason: &str| Event::Escaped {
        kind: id(kind),
        reason: id(reason),
        pr: None,
        checked: Checked::Unchecked,
        imported: None,
    };
    let (reply, events) = call(&mut ports, &["escape", "gap", "no", "review"], "");
    assert_eq!((reply.code, events), (Code::Done, vec![unchecked("gap", "no review")]));
    let why = ["escape-2 recorded: [gap] no review", "kind unchecked: no local self/escapes.toml"];
    assert!(why.iter().all(|part| reply.text.contains(part)), "{}", reply.text);
    let (reply, events) = call(&mut ports, &["escape", "gpa", "typo", "kept", "as", "given"], "");
    assert_eq!((reply.code, events), (Code::Done, vec![unchecked("gpa", "typo kept as given")]));
    let (reply, events) = call(&mut ports, &["escape", "unclassified", "old", "text"], "");
    assert_eq!((reply.code, events), (Code::Refused, Vec::new()), "{}", reply.text);
    let status = call(&mut ports, &["status"], "").0.text;
    assert!(status.contains("escape-2 [gap, unchecked] no review"), "{status}");
}

#[test]
fn escape_is_recorded_even_when_the_jsonl_log_does_not_parse() {
    let scratch = Scratch::new("garbled");
    let path = scratch.join("log.jsonl");
    std::fs::write(&path, "not a record\n").unwrap();
    let Ports { git, github, clock, build, .. } = given(Vec::new(), Vec::new());
    let mut ports = Ports { log: JsonlLog::at(path.clone()), git, github, clock, build };
    let reply = answer(parsed(&["escape", "gap", "log", "is", "garbled"]), &mut ports);
    let text = std::fs::read_to_string(&path).unwrap();
    assert_eq!(reply.code, Code::Done, "{}", reply.text);
    assert!(reply.text.contains("id and counts unavailable"), "{}", reply.text);
    let appended = r#"{"at":2,"event":"Escaped","kind":"gap","reason":"log is garbled"}"#;
    assert_eq!(text, format!("not a record\n{appended}\n"));
}

/// A log whose plain read is stale: only a read inside the transaction sees every record.
struct Lagging(MemoryLog);

impl Log for Lagging {
    fn place(&self) -> String {
        self.0.place()
    }

    fn standing(&self) -> Standing {
        self.0.standing()
    }

    fn set_aside(&mut self, escape: &UnverifiedEscape) -> Result<String, PortError> {
        self.0.set_aside(escape)
    }

    fn set_aside_escapes(&self) -> Result<UnverifiedEscapes, PortError> {
        self.0.set_aside_escapes()
    }

    fn import_set_aside<T>(
        &mut self,
        decide: impl FnOnce(
            Standing,
            Result<Vec<Record>, PortError>,
            Vec<SideLine>,
        ) -> (Vec<Record>, Option<Vec<SideLine>>, T),
    ) -> Result<T, PortError> {
        self.0.import_set_aside(decide)
    }

    fn mark(&mut self, marker: Record) -> Result<meta_core::Marking, PortError> {
        self.0.mark(marker)
    }

    fn read(&self) -> Result<Vec<Record>, PortError> {
        Ok(Vec::new())
    }

    fn transact<T>(
        &mut self,
        decide: impl FnOnce(Standing, Result<Vec<Record>, PortError>) -> (Vec<Record>, T),
    ) -> Result<T, PortError> {
        self.0.transact(decide)
    }
}

#[test]
fn escape_prints_the_id_it_gets_inside_the_locked_append() {
    let Ports { log, git, github, clock, build } =
        given(vec![escaped("gap", "one"), escaped("gap", "two")], Vec::new());
    let mut ports = Ports { log: Lagging(log), git, github, clock, build };
    let reply = answer(parsed(&["escape", "gap", "three"]), &mut ports);
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
    let status = call(&mut ports, &["status"], "").0.text;
    assert!(status.contains("P9 Untestable: UNDECIDED; no falsifier registered;"), "{status}");
}

#[test]
fn claim_by_an_agent_of_a_self_claimed_task_is_refused() {
    let self_claimed_task = || vec![opened("task-1"), self_claimed("task-1")];
    let histories = [self_claimed_task(), [self_claimed_task(), vec![linked("task-1", "7")]].concat()];
    histories.into_iter().for_each(|history| {
        let (reply, events) = call(&mut given(history, Vec::new()), &["claim", "task-1", "--agent", "bob"], "");
        assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
        assert!(reply.text.contains("self-claimed"), "{}", reply.text);
    });
}

#[test]
fn retire_of_a_deferred_task_is_refused() {
    let history = vec![opened("task-1"), Event::Deferred { task: id("task-1"), reason: id("later") }];
    let (reply, events) = call(&mut given(history, Vec::new()), &["retire", "task-1", "moot"], "");
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
    let (reply, added) = call(&mut ports, &["status"], "");
    let repeats = "most repeats of one escape kind 2";
    assert_eq!((reply.code, added), (Code::Done, vec![falsified("P2", Measure::RepeatsOfOneKind, repeats)]));
    assert!(reply.text.contains(&format!("falsified at unix 2 ({repeats})")), "{}", reply.text);
    let (reply, added) = call(&mut ports, &["status"], "");
    assert_eq!(added, Vec::new(), "{}", reply.text);
    assert!(reply.text.contains(&format!("falsified at unix 2 ({repeats})")), "{}", reply.text);
    ports.log.records.extend(gaps(3).into_iter().map(|event| Record { at: Timestamp::from_unix_seconds(3), event }));
    let (_, added) = call(&mut ports, &["status"], "");
    assert_eq!(added, vec![falsified("P2", Measure::OpenEscapes, "open escapes 6")]);
}

#[test]
fn status_keeps_a_latched_falsification_when_resolving_escapes_makes_the_prediction_hold_again() {
    let at = |seconds: u64, event: Event| Record { at: Timestamp::from_unix_seconds(seconds), event };
    let mut ports = given([finished_task(), unclassified(6)].concat(), Vec::new());
    let (reply, added) = call(&mut ports, &["status"], "");
    let open = "open escapes 6";
    assert_eq!(added, vec![falsified("P2", Measure::OpenEscapes, open)], "{}", reply.text);
    ports.log.records.extend([1, 2, 3].map(|number| at(3, resolved(&format!("escape-{number}"), "task-1"))));
    let (reply, added) = call(&mut ports, &["status"], "");
    assert_eq!(added, Vec::new(), "{}", reply.text);
    let holds = "P2 Gaps get fixed, not bypassed: holds; observed open escapes 3, most repeats of one escape kind 0;";
    assert!(reply.text.contains(holds), "{}", reply.text);
    assert!(reply.text.contains(&format!("falsified at unix 2 ({open})")), "{}", reply.text);
}

#[test]
fn status_shows_a_latched_falsification_read_from_the_log_and_latches_nothing_for_a_prediction_that_holds() {
    let history = [finished_task(), vec![falsified("P6", Measure::MedianCiRuns, "median CI runs per merged PR 4")]];
    let status = status_of(history.concat());
    let p6 = "P6 Little rework: holds; observed median CI runs per merged PR 1; expected Median at most 2; \
              falsified by Median above 3; measure: CI runs per merged PR; falsified at unix 1 (median CI runs per merged PR 4)\n";
    assert!(status.contains(p6), "{status}");
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
    let (reply, events) = call(&mut ports, &["resolve", "escape-1", "--historical", ONE_OFF], "");
    assert_eq!((reply.code, events), (Code::Done, vec![historical("escape-1", ONE_OFF)]));
    let (reply, _) = call(&mut ports, &["status"], "");
    assert!(reply.text.contains("open escapes: 1\n  escape-2 [gap] no review\n"), "{}", reply.text);
    assert!(reply.text.contains("P2 open escapes: 1 (1 closed as historical);"), "{}", reply.text);
    assert!(!reply.text.contains("escape-1"), "{}", reply.text);
    let (reply, _) = call(&mut ports, &["escape", "gap", "again"], "");
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
        let (reply, events) = call(&mut given(history, Vec::new()), &["resolve", args, "--historical", ONE_OFF], "");
        assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
        assert!(reply.text.contains(why), "{}", reply.text);
    });
}

#[test]
fn resolve_historical_correcting_moves_a_task_resolution_to_historical_and_status_shows_both() {
    let history = [finished_task(), vec![escaped("gap", "no retire"), resolved("escape-1", "task-1")]].concat();
    let mut ports = given(history, Vec::new());
    let (reply, events) = call(&mut ports, &["resolve", "escape-1", "--historical", ONE_OFF, "--correcting"], "");
    assert_eq!((reply.code, events), (Code::Done, vec![corrected("escape-1", ONE_OFF)]));
    assert!(ports.log.records.iter().any(|record| record.event == resolved("escape-1", "task-1")));
    let (reply, _) = call(&mut ports, &["status"], "");
    let shown = format!("escape-1 resolved against task-1, corrected to historical: {ONE_OFF}");
    assert!(reply.text.contains(&shown) && reply.text.contains("open escapes: 0"), "{}", reply.text);
    assert!(reply.text.contains("P2 open escapes: 0 (1 closed as historical);"), "{}", reply.text);
    let (reply, events) = call(&mut ports, &["resolve", "escape-1", "--historical", ONE_OFF, "--correcting"], "");
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
        let (reply, events) = call(
            &mut given(history, Vec::new()),
            &["resolve", "escape-1", "--historical", ONE_OFF, "--correcting"],
            "",
        );
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
        let (reply, _) = call(&mut ports, &["status"], "");
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
        let (reply, _) = call(&mut given(history, Vec::new()), &["status"], "");
        assert_eq!(reply.code, Code::Failed, "{}", reply.text);
    });
}

const BRANCH_BUILD_REFUSAL: &str = "you are running a branch build";
const MAIN_BUILD_NEXT: &str = "Next: use a main build, or set XDG_DATA_HOME to a scratch data dir.";

/// The default data dir's log, not yet marked, written by `build`.
fn on_real_log(events: Vec<Event>, pulls: Vec<(&str, GhReplies)>, build: Build) -> Memory {
    Ports { build, ..given(events, pulls) }.real()
}

trait Real {
    fn real(self) -> Self;
}

impl Real for Memory {
    fn real(mut self) -> Self {
        self.log.place = Place::Default;
        self
    }
}

/// A commit not on origin/main, edits to tracked files, and a commit whose ancestry cannot be read.
fn unverified_builds() -> [(Build, &'static str); 3] {
    [
        (Build::commit(id(BRANCH_BUILD)), "built from b4a2c40, which is not on origin/main"),
        (
            Build::embedded(None, Some("c0ffee0"), None),
            "built from c0ffee0 with edits to tracked files; to clear it, commit or stash them, then rebuild",
        ),
        (Build::commit(id("d00d1e5")), "cannot tell whether d00d1e5 is on origin/main: no recorded ancestry"),
    ]
}

#[test]
fn real_log_refuses_every_ledger_write_from_a_build_not_on_main_and_merges_nothing() {
    let green = || vec![("7", pull(PINNED, &[Outcome::Green]))];
    let cases: [(Vec<Event>, &[&str]); 7] = [
        (vec![opened("task-1")], &["open", "Fix", "the", "hook"]),
        (vec![opened("task-1")], &["claim", "task-1", "--agent", "alice"]),
        (claimed_task(), &["link", "task-1", "7"]),
        (linked_task(), &["merge", "7"]),
        (vec![opened("task-1")], &["retire", "task-1", "moot"]),
        (vec![opened("task-1")], &["defer", "task-1", "later"]),
        ([finished_task(), vec![escaped("gap", "one")]].concat(), &["resolve", "escape-1", "task-1"]),
    ];
    unverified_builds().into_iter().for_each(|(build, why)| {
        cases.iter().for_each(|(history, args)| {
            let mut ports = on_real_log(history.clone(), green(), build.clone());
            let (reply, events) = call(&mut ports, args, "");
            assert_eq!((reply.code, events), (Code::Refused, Vec::new()), "{args:?}: {}", reply.text);
            let said = [BRANCH_BUILD_REFUSAL, why, MAIN_BUILD_NEXT];
            assert!(said.iter().all(|part| reply.text.contains(part)), "{args:?}: {}", reply.text);
            assert!(ports.github.merged.is_empty(), "{args:?}");
        });
    });
}

#[test]
fn real_log_denies_agent_starts_resumes_and_bindings_from_a_build_not_on_main_failing_closed() {
    let resume = send(AGENT_ID, serde_json::json!("continue"), Some("lease-1"));
    let cases: [(&[&str], Vec<Event>, String, &str); 3] = [
        (&["hook", "pre-tool-use"], claimed_task(), agent("lease-1"), DENY),
        (&["hook", "pre-tool-use"], bound(), resume, DENY),
        (
            &["hook", "post-tool-use"],
            [claimed_task(), vec![used("lease-1")]].concat(),
            agent_ran("Agent", "lease-1", AGENT_ID),
            r#""decision":"block""#,
        ),
    ];
    unverified_builds().into_iter().for_each(|(build, why)| {
        cases.iter().for_each(|(args, history, input, blocked)| {
            let (reply, events) = call(&mut on_real_log(history.clone(), Vec::new(), build.clone()), args, input);
            assert_eq!((reply.code, events), (Code::Done, Vec::new()), "{}", reply.text);
            let said = [*blocked, "fail closed", BRANCH_BUILD_REFUSAL, why];
            assert!(said.iter().all(|part| reply.text.contains(part)), "{}", reply.text);
        });
    });
}

#[test]
fn real_log_still_answers_reads_from_a_build_not_on_main_and_latches_nothing() {
    let gaps: Vec<Event> = (0..3).map(|number| escaped("gap", &format!("gap {number}"))).collect();
    unverified_builds().into_iter().for_each(|(build, why)| {
        let (reply, events) = call(&mut on_real_log(claimed_task(), Vec::new(), build.clone()), &["status"], "");
        assert_eq!((reply.code, events), (Code::Done, Vec::new()), "{}", reply.text);
        assert!(
            reply.text.contains("task-1 claimed by alice") && !reply.text.contains("not recorded"),
            "{}",
            reply.text
        );
        let (reply, events) = call(&mut on_real_log(gaps.clone(), Vec::new(), build), &["status"], "");
        assert_eq!((reply.code, events), (Code::Done, Vec::new()), "{}", reply.text);
        assert!(reply.text.contains("FALSIFIED") && reply.text.contains("not recorded: "), "{}", reply.text);
        assert!(reply.text.contains(why), "{}", reply.text);
        let bash = json_input("Bash", "command", "ls");
        let (reply, events) = call(
            &mut on_real_log(claimed_task(), Vec::new(), Build::commit(id(BRANCH_BUILD))),
            &["hook", "pre-tool-use"],
            &bash,
        );
        assert_eq!((reply, events), (Answer { text: ALLOW.to_owned(), code: Code::Done }, Vec::new()));
    });
}

#[test]
fn real_log_sets_aside_an_escape_from_a_build_not_on_main_and_appends_nothing() {
    unverified_builds().into_iter().for_each(|(build, why)| {
        let mut ports = on_real_log(vec![claimed("task-1")], Vec::new(), build);
        let (reply, events) = call(&mut ports, &["escape", "gap", "the", "hook", "denies", "every", "start"], "");
        assert_eq!((reply.code, events), (Code::Done, Vec::new()), "{}", reply.text);
        let said = ["unverified escape recorded: [gap] the hook denies every start", "kept in memory side file", why];
        assert!(said.iter().all(|part| reply.text.contains(part)), "{}", reply.text);
        assert_eq!(ports.log.records.len(), 1);
        assert_eq!(ports.log.set_aside.len(), 1);
    });
}

#[test]
fn real_log_takes_every_write_from_a_build_on_main() {
    let mut ports = on_real_log(claimed_task(), Vec::new(), Build::commit(id(MAIN_BUILD)));
    let (reply, events) = call(&mut ports, &["open", "Fix", "the", "hook"], "");
    assert_eq!((reply.code, events), (Code::Done, vec![opened("task-2")]), "{}", reply.text);
    let (reply, events) = call(&mut ports, &["hook", "pre-tool-use"], &agent("lease-1"));
    assert_eq!((reply.text, events), (ALLOW.to_owned(), vec![used("lease-1")]));
    let (reply, events) = call(&mut ports, &["escape", "gap", "from", "main"], "");
    assert_eq!((reply.code, events), (Code::Done, vec![escaped("gap", "from main")]), "{}", reply.text);
}

#[test]
fn a_scratch_log_takes_every_write_from_any_build_with_no_ancestry_check() {
    unverified_builds().into_iter().for_each(|(build, _)| {
        let mut ports = Ports { build, ..given(claimed_task(), Vec::new()) };
        ports.git.on_main.clear();
        let (reply, events) = call(&mut ports, &["open", "Fix", "the", "hook"], "");
        assert_eq!((reply.code, events), (Code::Done, vec![opened("task-2")]), "{}", reply.text);
        let (reply, events) = call(&mut ports, &["hook", "pre-tool-use"], &agent("lease-1"));
        assert_eq!((reply.text, events), (ALLOW.to_owned(), vec![used("lease-1")]));
        let (reply, events) = call(&mut ports, &["escape", "gap", "scratch"], "");
        assert_eq!((reply.code, events), (Code::Done, vec![escaped("gap", "scratch")]), "{}", reply.text);
    });
}

/// A log that starts with the `RealLog` marker, at no default place: reached by another HOME, a
/// hard link, a symlink or with HOME unset. The JSONL port's tests reach it each of those ways.
fn on_marked_log(events: Vec<Event>, build: Build) -> Memory {
    let marked = [vec![Event::RealLog], events].concat();
    Ports { build, ..given(marked, Vec::new()) }
}

#[test]
fn a_marked_log_anywhere_refuses_every_write_from_a_build_not_on_main_and_sets_escapes_aside() {
    unverified_builds().into_iter().for_each(|(build, why)| {
        let (reply, events) =
            call(&mut on_marked_log(vec![opened("task-1")], build.clone()), &["defer", "task-1", "later"], "");
        assert_eq!((reply.code, events), (Code::Refused, Vec::new()), "{}", reply.text);
        assert!(reply.text.contains(why), "{}", reply.text);
        let (reply, events) =
            call(&mut on_marked_log(claimed_task(), build.clone()), &["hook", "pre-tool-use"], &agent("lease-1"));
        assert_eq!(events, Vec::new(), "{}", reply.text);
        assert!(reply.text.contains(DENY), "{}", reply.text);
        let (reply, events) =
            call(&mut on_marked_log(Vec::new(), build), &["escape", "gap", "from", "a", "branch"], "");
        assert_eq!((reply.code, events), (Code::Done, Vec::new()), "{}", reply.text);
    });
    let (reply, events) = call(
        &mut on_marked_log(vec![opened("task-1")], Build::commit(id(MAIN_BUILD))),
        &["defer", "task-1", "later"],
        "",
    );
    assert_eq!((reply.code, events.len()), (Code::Done, 1), "{}", reply.text);
}

#[test]
fn a_marked_jsonl_log_at_a_scratch_path_is_still_refused_to_a_branch_build() {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("marked-{nanos}.jsonl"));
    let marked = "{\"at\":1,\"event\":\"RealLog\"}\n{\"at\":1,\"event\":\"Opened\",\"task\":1,\"title\":\"Fix\"}\n";
    std::fs::write(&path, marked).unwrap();
    let Ports { git, github, clock, .. } = given(Vec::new(), Vec::new());
    let mut ports =
        Ports { log: JsonlLog::at(path.clone()), git, github, clock, build: Build::commit(id(BRANCH_BUILD)) };
    let reply = answer(parsed(&["defer", "task-1", "later"]), &mut ports);
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
    assert_eq!((reply.code, text.as_str()), (Code::Refused, marked), "{}", reply.text);
}

#[test]
fn mark_real_log_puts_the_marker_first_once_and_only_from_a_main_build() {
    unverified_builds().into_iter().for_each(|(build, why)| {
        let mut ports = Ports { build, ..given(claimed_task(), Vec::new()) };
        let reply = answer(parsed(&["mark-real-log"]), &mut ports);
        assert_eq!(reply.code, Code::Refused, "{}", reply.text);
        assert!(reply.text.contains(why), "{}", reply.text);
        assert_eq!(ports.log.records.len(), 3);
    });
    let mut ports = on_real_log(claimed_task(), Vec::new(), Build::commit(id(MAIN_BUILD)));
    let first = answer(parsed(&["mark-real-log"]), &mut ports);
    let again = answer(parsed(&["mark-real-log"]), &mut ports);
    let events: Vec<Event> = ports.log.records.iter().map(|record| record.event.clone()).collect();
    assert_eq!((first.code, again.code), (Code::Done, Code::Done), "{}{}", first.text, again.text);
    assert!(first.text.contains("marked memory log as the real log"), "{}", first.text);
    assert!(again.text.contains("already marked"), "{}", again.text);
    assert_eq!(events, [vec![Event::RealLog], claimed_task()].concat());
    let status = call(&mut ports, &["status"], "");
    assert!(status.0.text.contains("task-1 claimed by alice"), "{}", status.0.text);
}

#[test]
fn claim_brief_tells_the_agent_to_link_with_a_main_build_and_keep_branch_builds_on_a_scratch_log() {
    let (reply, _) = call(&mut given(vec![opened("task-1")], Vec::new()), &["claim", "task-1", "--agent", "alice"], "");
    let steps = [
        "git worktree add --detach ../task-1-main origin/main",
        "cargo build --release --manifest-path ../task-1-main/Cargo.toml --target-dir ../task-1-main/target",
        "../task-1-main/target/release/meta link task-1 <pr>",
        "XDG_DATA_HOME set to a scratch dir",
    ];
    assert!(steps.iter().all(|step| reply.text.contains(step)), "{}", reply.text);
}

const SIDE_FILE: &str = "escapes.unverified.jsonl";
const MARKER_LINE: &str = "{\"at\":1,\"event\":\"RealLog\"}\n";
const MARK_THE_LOG: &str = "run meta mark-real-log with a main build";
const MARKED_REFUSAL: &str = "this log is marked as the real log; use a main build";
const SCRATCH_ADVICE: &str = "XDG_DATA_HOME";

/// The JSONL log in `dir`, holding only the marker, run by `build`.
fn on_marked_jsonl(dir: &std::path::Path, build: Build) -> Ports<JsonlLog, RecordedGit, RecordedGitHub, FakeClock> {
    let path = dir.join("log.jsonl");
    std::fs::write(&path, MARKER_LINE).unwrap();
    let Ports { git, github, clock, .. } = given(Vec::new(), Vec::new());
    Ports { log: JsonlLog::at(path), git, github, clock, build }
}

#[test]
fn an_escape_from_a_build_not_on_main_goes_to_the_side_file_and_never_into_the_marked_log() {
    unverified_builds().into_iter().for_each(|(build, why)| {
        let scratch = Scratch::new("side-write");
        let mut ports = on_marked_jsonl(&scratch, build);
        let reply = answer(parsed(&["escape", "gap", "the", "hook", "denies", "every", "start"]), &mut ports);
        assert_eq!(reply.code, Code::Done, "{}", reply.text);
        let said = ["unverified", SIDE_FILE, why];
        assert!(said.iter().all(|part| reply.text.contains(part)), "{}", reply.text);
        assert_eq!(std::fs::read_to_string(scratch.join("log.jsonl")).unwrap(), MARKER_LINE);
        let side = std::fs::read_to_string(scratch.join(SIDE_FILE)).unwrap();
        let entry: serde_json::Value = serde_json::from_str(&side).unwrap();
        assert_eq!(entry["kind"], "gap", "{side}");
        assert_eq!(entry["reason"], "the hook denies every start", "{side}");
    });
}

#[test]
fn a_side_file_that_cannot_be_written_fails_the_escape_and_leaves_the_log_alone() {
    let scratch = Scratch::new("side-blocked");
    let mut ports = on_marked_jsonl(&scratch, Build::commit(id(BRANCH_BUILD)));
    std::fs::create_dir(scratch.join(SIDE_FILE)).unwrap();
    let reply = answer(parsed(&["escape", "gap", "from", "a", "branch"]), &mut ports);
    assert_eq!(reply.code, Code::Failed, "{}", reply.text);
    assert!(reply.text.contains(SIDE_FILE), "{}", reply.text);
    assert_eq!(std::fs::read_to_string(scratch.join("log.jsonl")).unwrap(), MARKER_LINE);
}

#[test]
fn a_symlinked_log_leads_to_the_side_file_beside_the_log_it_reaches() {
    let scratch = Scratch::new("side-symlink");
    let (real, reach) = (scratch.join("real"), scratch.join("reach"));
    std::fs::create_dir_all(&reach).unwrap();
    std::fs::create_dir_all(&real).unwrap();
    let mut ports = on_marked_jsonl(&real, Build::commit(id(BRANCH_BUILD)));
    std::os::unix::fs::symlink(real.join("log.jsonl"), reach.join("log.jsonl")).unwrap();
    ports.log = JsonlLog::at(reach.join("log.jsonl"));
    assert_eq!(answer(parsed(&["escape", "gap", "linked"]), &mut ports).code, Code::Done);
    assert!(real.join(SIDE_FILE).is_file() && !reach.join(SIDE_FILE).exists());
}

#[test]
fn status_shows_side_file_escapes_marked_unverified_and_p2_counts_them() {
    let scratch = Scratch::new("side-status");
    let mut ports = on_marked_jsonl(&scratch, Build::commit(id(BRANCH_BUILD)));
    (0..3).for_each(|_| {
        let reply = answer(parsed(&["escape", "gap", "from", "a", "branch"]), &mut ports);
        assert_eq!(reply.code, Code::Done, "{}", reply.text);
    });
    ports.build = Build::commit(id(MAIN_BUILD));
    let status = answer(parsed(&["status"]), &mut ports);
    assert_eq!(status.code, Code::Done, "{}", status.text);
    let said = [
        "open escapes: 3\n",
        "unverified [gap] from a branch\n",
        "P2 open escapes: 3 ",
        "most repeats of one kind: 2 (gap)",
        "P2 Gaps get fixed, not bypassed: FALSIFIED; observed open escapes 3, most repeats of one escape kind 2;",
    ];
    assert!(said.iter().all(|part| status.text.contains(part)), "{}", status.text);
}

#[test]
fn status_reads_the_side_file_leniently_counting_unreadable_lines_and_never_failing() {
    let scratch = Scratch::new("side-lenient");
    let mut ports = on_marked_jsonl(&scratch, Build::commit(id(MAIN_BUILD)));
    let known = "{\"at\":3,\"kind\":\"gap\",\"reason\":\"kept\",\"future\":{\"field\":1}}\n";
    let lines: [&[u8]; 3] = [known.as_bytes(), b"not a record\n", b"\xff\xfe\n"];
    std::fs::write(scratch.join(SIDE_FILE), lines.concat()).unwrap();
    let status = answer(parsed(&["status"]), &mut ports);
    assert_eq!(status.code, Code::Done, "{}", status.text);
    let said = ["unverified [gap] kept\n", "P2 open escapes: 1 ", "2 unreadable lines"];
    assert!(said.iter().all(|part| status.text.contains(part)), "{}", status.text);
}

#[test]
fn status_survives_a_side_file_it_cannot_read_at_all_and_says_so() {
    let scratch = Scratch::new("side-unreadable");
    let mut ports = on_marked_jsonl(&scratch, Build::commit(id(MAIN_BUILD)));
    std::fs::create_dir(scratch.join(SIDE_FILE)).unwrap();
    let status = answer(parsed(&["status"]), &mut ports);
    assert_eq!(status.code, Code::Done, "{}", status.text);
    assert!(status.text.contains("unverified escapes unavailable: "), "{}", status.text);
}

#[test]
fn status_warns_while_the_default_data_dirs_log_is_unmarked_and_not_after() {
    let builds = [Build::commit(id(MAIN_BUILD)), Build::commit(id(BRANCH_BUILD))];
    builds.into_iter().for_each(|build| {
        [Vec::new(), claimed_task()].into_iter().for_each(|history| {
            let mut ports = on_real_log(history, Vec::new(), build.clone());
            let (reply, _) = call(&mut ports, &["status"], "");
            assert_eq!(reply.code, Code::Done, "{}", reply.text);
            assert!(reply.text.contains(MARK_THE_LOG), "{}", reply.text);
        });
    });
    let mut marked = on_real_log(claimed_task(), Vec::new(), Build::commit(id(MAIN_BUILD)));
    assert_eq!(call(&mut marked, &["mark-real-log"], "").0.code, Code::Done);
    let scratch = call(&mut given(claimed_task(), Vec::new()), &["status"], "").0.text;
    let after = call(&mut marked, &["status"], "").0.text;
    assert!(!after.contains(MARK_THE_LOG), "{after}");
    assert!(!scratch.contains(MARK_THE_LOG), "{scratch}");
}

#[test]
fn a_marked_log_reached_through_a_scratch_dir_is_refused_without_advising_a_scratch_dir() {
    unverified_builds().into_iter().for_each(|(build, why)| {
        let scratch = Scratch::new("marked-next");
        let cases: [(&[&str], &str); 3] = [
            (&["open", "Fix", "the", "hook"], ""),
            (&["hook", "pre-tool-use"], &agent("lease-1")),
            (&["escape", "gap", "from", "a", "branch"], ""),
        ];
        cases.into_iter().for_each(|(args, stdin)| {
            let mut ports = on_marked_jsonl(&scratch, build.clone());
            let reply = answer(Call::parse(&command(args), || stdin.to_owned()).unwrap(), &mut ports);
            let said = [MARKED_REFUSAL, why];
            assert!(said.iter().all(|part| reply.text.contains(part)), "{args:?}: {}", reply.text);
            assert!(!reply.text.contains(SCRATCH_ADVICE), "{args:?}: {}", reply.text);
        });
    });
}

#[test]
fn a_marked_log_in_the_default_data_dir_still_advises_a_scratch_dir_for_a_branch_build() {
    let mut ports = on_real_log(vec![Event::RealLog], Vec::new(), Build::commit(id(BRANCH_BUILD)));
    let (reply, _) = call(&mut ports, &["open", "Fix", "the", "hook"], "");
    assert_eq!(reply.code, Code::Refused, "{}", reply.text);
    assert!(reply.text.contains(MAIN_BUILD_NEXT), "{}", reply.text);
}

/// A log whose standing, read before the append, is stale: it says scratch, while the log inside
/// the lock starts with the marker.
struct Unmarked(MemoryLog);

impl Log for Unmarked {
    fn place(&self) -> String {
        self.0.place()
    }

    fn standing(&self) -> Standing {
        Standing::Scratch
    }

    fn set_aside(&mut self, escape: &UnverifiedEscape) -> Result<String, PortError> {
        self.0.set_aside(escape)
    }

    fn set_aside_escapes(&self) -> Result<UnverifiedEscapes, PortError> {
        self.0.set_aside_escapes()
    }

    fn import_set_aside<T>(
        &mut self,
        decide: impl FnOnce(
            Standing,
            Result<Vec<Record>, PortError>,
            Vec<SideLine>,
        ) -> (Vec<Record>, Option<Vec<SideLine>>, T),
    ) -> Result<T, PortError> {
        self.0.import_set_aside(decide)
    }

    fn mark(&mut self, marker: Record) -> Result<meta_core::Marking, PortError> {
        self.0.mark(marker)
    }

    fn read(&self) -> Result<Vec<Record>, PortError> {
        self.0.read()
    }

    fn transact<T>(
        &mut self,
        decide: impl FnOnce(Standing, Result<Vec<Record>, PortError>) -> (Vec<Record>, T),
    ) -> Result<T, PortError> {
        self.0.transact(decide)
    }
}

#[test]
fn a_branch_build_that_read_scratch_before_the_log_was_marked_appends_nothing_inside_the_lock() {
    let cases: [(&[&str], String); 3] = [
        (&["defer", "task-1", "later"], String::new()),
        (&["hook", "pre-tool-use"], agent("lease-1")),
        (&["escape", "gap", "from", "a", "stale", "build"], String::new()),
    ];
    cases.into_iter().for_each(|(args, stdin)| {
        let Ports { log, git, github, clock, .. } = on_marked_log(claimed_task(), Build::commit(id(BRANCH_BUILD)));
        let before = log.records.len();
        let mut ports = Ports { log: Unmarked(log), git, github, clock, build: Build::commit(id(BRANCH_BUILD)) };
        let reply = answer(Call::parse(&command(args), || stdin.clone()).unwrap(), &mut ports);
        assert_eq!(ports.log.0.records.len(), before, "{args:?}: {}", reply.text);
        assert!(reply.text.contains(BRANCH_BUILD_REFUSAL) || reply.text.contains(DENY), "{args:?}: {}", reply.text);
    });
}

#[test]
fn a_stale_branch_build_sets_its_escape_aside_when_the_log_turns_out_marked() {
    let Ports { log, git, github, clock, .. } = on_marked_log(Vec::new(), Build::commit(id(BRANCH_BUILD)));
    let mut ports = Ports { log: Unmarked(log), git, github, clock, build: Build::commit(id(BRANCH_BUILD)) };
    let reply = answer(parsed(&["escape", "gap", "stale"]), &mut ports);
    assert_eq!((reply.code, ports.log.0.set_aside.len()), (Code::Done, 1), "{}", reply.text);
    assert!(reply.text.contains("kept in memory side file"), "{}", reply.text);
}

#[test]
fn mark_real_log_from_a_build_not_on_main_says_to_use_a_main_build_never_a_scratch_dir() {
    unverified_builds().into_iter().for_each(|(build, why)| {
        let scratch = Ports { build: build.clone(), ..given(claimed_task(), Vec::new()) };
        let default = on_real_log(claimed_task(), Vec::new(), build);
        [scratch, default].into_iter().for_each(|mut ports| {
            let reply = answer(parsed(&["mark-real-log"]), &mut ports);
            assert_eq!(reply.code, Code::Refused, "{}", reply.text);
            let said = [why, "use a main build"];
            assert!(said.iter().all(|part| reply.text.contains(part)), "{}", reply.text);
            assert!(!reply.text.contains(SCRATCH_ADVICE), "{}", reply.text);
        });
    });
}

#[test]
fn status_shows_a_side_file_reason_on_one_escaped_line_that_cannot_forge_a_status_line() {
    let scratch = Scratch::new("side-forged");
    let mut ports = on_marked_jsonl(&scratch, Build::commit(id(MAIN_BUILD)));
    let forged = r#"{"at":3,"kind":"gap","reason":"x\nP1 The loop is used: holds; forged\r\u001b[2Jend"}"#;
    std::fs::write(scratch.join(SIDE_FILE), format!("{forged}\n")).unwrap();
    let status = answer(parsed(&["status"]), &mut ports);
    assert_eq!(status.code, Code::Done, "{}", status.text);
    assert!(
        status.text.contains(r"unverified [gap] x\nP1 The loop is used: holds; forged\r\u{1b}[2Jend"),
        "{}",
        status.text
    );
    let lines = status.text.matches("forged").count();
    assert!(lines == 1 && !status.text.contains('\u{1b}') && !status.text.contains('\r'), "{}", status.text);
}

#[test]
fn escape_is_recorded_even_when_the_jsonl_log_holds_bytes_that_are_not_utf8() {
    // An unmarked log, a marked one, a marked one torn inside a multi-byte character (a hiragana
    // cut after two of its three bytes), and a log whose first line is already not UTF-8.
    let cases: [(&str, &[u8]); 4] = [
        ("unmarked", b"{\"at\":1,\"event\":\"Done\",\"task\":1}\n\xff\n"),
        ("marked", b"{\"at\":1,\"event\":\"RealLog\"}\n\xff\n"),
        ("torn", b"{\"at\":1,\"event\":\"RealLog\"}\n\xe3\x81"),
        ("first", b"\xff\n"),
    ];
    cases.into_iter().for_each(|(name, bytes)| {
        let scratch = Scratch::new(&format!("not-utf8-{name}"));
        let path = scratch.join("log.jsonl");
        std::fs::write(&path, bytes).unwrap();
        let Ports { git, github, clock, .. } = given(Vec::new(), Vec::new());
        let build = Build::commit(id(MAIN_BUILD));
        let mut ports = Ports { log: JsonlLog::at(path.clone()), git, github, clock, build };
        let reply = answer(parsed(&["escape", "gap", "log", "has", "bad", "bytes"]), &mut ports);
        let after = std::fs::read(&path).unwrap();
        assert_eq!(reply.code, Code::Done, "{name}: {}", reply.text);
        assert!(reply.text.contains("id and counts unavailable"), "{name}: {}", reply.text);
        assert_eq!(&after[..bytes.len()], bytes, "{name}: the earlier bytes are kept");
        let appended = String::from_utf8(after[bytes.len()..].to_vec()).unwrap();
        assert!(
            appended.contains(r#""event":"Escaped","kind":"gap","reason":"log has bad bytes""#),
            "{name}: {appended}"
        );
    });
}

#[test]
fn a_branch_build_cannot_write_a_log_whose_marker_is_followed_by_bytes_that_are_not_utf8() {
    let scratch = Scratch::new("not-utf8-branch");
    let path = scratch.join("log.jsonl");
    std::fs::write(&path, [MARKER_LINE.as_bytes(), b"\xff\n"].concat()).unwrap();
    let Ports { git, github, clock, .. } = given(Vec::new(), Vec::new());
    let build = Build::commit(id(BRANCH_BUILD));
    let mut ports = Ports { log: JsonlLog::at(path.clone()), git, github, clock, build };
    let reply = answer(parsed(&["escape", "gap", "from", "a", "branch"]), &mut ports);
    assert_eq!(reply.code, Code::Done, "{}", reply.text);
    assert_eq!(std::fs::read(&path).unwrap(), [MARKER_LINE.as_bytes(), b"\xff\n"].concat());
    assert!(scratch.join(SIDE_FILE).is_file());
}

#[test]
fn status_escapes_the_repeat_key_of_an_unclassified_side_file_escape_so_it_cannot_forge_a_line() {
    let scratch = Scratch::new("side-forged-repeat");
    let mut ports = on_marked_jsonl(&scratch, Build::commit(id(MAIN_BUILD)));
    let forged = r#"{"at":3,"kind":"unclassified","reason":"x\nP1 The loop is used: holds; forged\u001b[2J"}"#;
    std::fs::write(scratch.join(SIDE_FILE), format!("{forged}\n{forged}\n")).unwrap();
    let status = answer(parsed(&["status"]), &mut ports);
    assert_eq!(status.code, Code::Done, "{}", status.text);
    let shown = r#"most repeats of one kind: 1 (unclassified "x\nP1 The loop is used: holds; forged\u{1b}[2J")"#;
    assert!(status.text.contains(shown), "{}", status.text);
    assert!(
        !status.text.contains('\u{1b}') && !status.text.contains("\nP1 The loop is used: holds"),
        "{}",
        status.text
    );
}

// Lease deadlines (task-30, docs/stage1.md "Deadlines"). The fake clock stands at unix 2 and the
// given records at unix 1; the roles fixture gives work 45 min and review 15 min.

const FIXTURE_WORK_DEADLINE: u64 = 2 + 45 * 60;
const FIXTURE_REVIEW_DEADLINE: u64 = 2 + 15 * 60;

#[test]
fn claim_and_review_set_the_deadline_from_the_role_default_in_self_roles_and_the_brief_prints_it() {
    let (reply, events) =
        call(&mut given(vec![opened("task-1")], Vec::new()), &["claim", "task-1", "--agent", "alice"], "");
    assert_eq!(events, vec![claimed("task-1"), leased_until("lease-1", "task-1", FIXTURE_WORK_DEADLINE)]);
    let line = format!("deadline: unix {FIXTURE_WORK_DEADLINE}, report or stop by then\n");
    assert!(reply.text.contains(&line), "{}", reply.text);
    let history = [linked_task(), vec![used("lease-1")]].concat();
    let (reply, events) =
        call(&mut given(history, vec![("7", pull(MOVED, &[]))]), &["review", "task-1", "--agent", "rev"], "");
    assert_eq!(events, vec![review_leased_until("lease-2", MOVED, FIXTURE_REVIEW_DEADLINE)]);
    let line = format!("deadline: unix {FIXTURE_REVIEW_DEADLINE}, report or stop by then\n");
    assert!(reply.text.contains(&line), "{}", reply.text);
    let mut unreadable = given(vec![opened("task-1")], Vec::new());
    unreadable.git.files.remove(&SelfFile::Roles);
    let (reply, events) = call(&mut unreadable, &["claim", "task-1", "--agent", "alice"], "");
    assert_eq!((reply.code, events), (Code::Failed, Vec::new()), "{}", reply.text);
    assert!(reply.text.contains("self/roles.toml"), "{}", reply.text);
}

#[test]
fn claim_and_review_take_a_deadline_override_in_minutes_and_self_claims_get_no_deadline() {
    let (reply, events) = call(
        &mut given(vec![opened("task-1")], Vec::new()),
        &["claim", "task-1", "--agent", "alice", "--deadline", "10"],
        "",
    );
    assert_eq!(events, vec![claimed("task-1"), leased_until("lease-1", "task-1", 2 + 600)]);
    assert!(reply.text.contains("deadline: unix 602, report or stop by then"), "{}", reply.text);
    let history = [linked_task(), vec![used("lease-1")]].concat();
    let mut ports = given(history, vec![("7", pull(MOVED, &[]))]);
    let (_, events) = call(&mut ports, &["review", "task-1", "--agent", "rev", "--deadline", "5"], "");
    assert_eq!(events, vec![review_leased_until("lease-2", MOVED, 2 + 300)]);
    let (reply, events) = call(&mut given(vec![opened("task-1")], Vec::new()), &["claim", "task-1", "--self"], "");
    assert_eq!((reply.code, events), (Code::Done, vec![self_claimed("task-1")]));
    assert!(reply.text.contains("deadline: none"), "{}", reply.text);
    let malformed = [
        &["claim", "task-1", "--self", "--deadline", "10"][..],
        &["claim", "task-1", "--agent", "a", "--deadline", "0"],
    ];
    let more = [
        &["claim", "task-1", "--agent", "a", "--deadline", "1.5"][..],
        &["review", "task-1", "--agent", "a", "--deadline", "x"],
    ];
    malformed.into_iter().chain(more).for_each(|args| {
        let reply = Call::parse(&command(args), String::new).unwrap_err();
        assert!(reply.code == Code::Failed && reply.text.contains("usage:"), "{args:?}: {}", reply.text);
    });
}

#[test]
fn the_started_record_carries_the_deadline_counted_from_the_start_time() {
    let history =
        [vec![opened("task-1"), claimed("task-1"), leased_until("lease-1", "task-1", 1 + 2700)], vec![used("lease-1")]]
            .concat();
    let mut ports = given(history, Vec::new());
    ports.clock.pause(Span::of_seconds(100));
    let (reply, events) = call(&mut ports, &["hook", "post-tool-use"], &agent_ran("Agent", "lease-1", AGENT_ID));
    let deadline = Some(Timestamp::from_unix_seconds(102 + 2700));
    let started = Event::Started { lease: id("lease-1"), agent: id(AGENT_ID), deadline };
    assert_eq!((reply.text.as_str(), events), (ALLOW, vec![started]));
    ports.clock.pause(Span::of_seconds(2750 - 102));
    let (status, _) = call(&mut ports, &["status"], "");
    assert!(
        status.text.contains("overdue leases (flagged)\n  (none)\n"),
        "past the lease's own deadline: {}",
        status.text
    );
    ports.clock.pause(Span::of_seconds(2803 - 2750));
    let (status, _) = call(&mut ports, &["status"], "");
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
    let adjacent = "leases resumed 3 or more times (flagged)\n  (none)\noverdue leases (flagged)\n";
    assert!(resumes.contains(adjacent), "{resumes}");
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
    assert_eq!(line, r#"{"at":1,"event":"Leased","lease":1,"task":1,"agent":"alice","deadline":9}"#);
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
    let (reply, _) = call(&mut ports, &["status"], "");
    assert!(reply.text.contains("P1 The loop is used: FALSIFIED; observed unrecorded bypasses 1;"), "{}", reply.text);
    assert!(reply.text.contains("bypassed, from outside-start escapes: 1;"), "{}", reply.text);
}

// Task 33: waiting on the owner pauses deadlines, and the idle alarm (docs/stage1.md "Waiting on the owner").

fn waiting(task: &str, question: &str) -> Event {
    Event::WaitingOnOwner { task: id(task), question: id(question) }
}

fn answered(task: &str, answer: &str) -> Event {
    Event::OwnerAnswered { task: id(task), answer: id(answer) }
}

const IDLE: &str = "idle: no live work; stalled: ";

#[test]
fn wait_records_the_question_and_status_lists_the_task_under_waiting_on_the_owner() {
    let mut ports = given(claimed_task(), Vec::new());
    let (reply, events) = call(&mut ports, &["wait", "task-1", "--owner", "Merge", "the", "mega", "PR?"], "");
    assert_eq!((reply.code, events), (Code::Done, vec![waiting("task-1", "Merge the mega PR?")]));
    let (status, _) = call(&mut ports, &["status"], "");
    let listed = "waiting on the owner (flagged)\n  task-1: Merge the mega PR? (waiting 0 min)\n";
    assert!(status.text.contains(listed), "{}", status.text);
    let (reply, events) = call(&mut ports, &["wait", "task-1", "--owner", "again"], "");
    assert_eq!((reply.code, events), (Code::Refused, Vec::new()));
    assert!(reply.text.contains("already waiting"), "{}", reply.text);
    let (reply, events) = call(&mut ports, &["unwait", "task-1", "Yes,", "merge", "it"], "");
    assert_eq!((reply.code, events), (Code::Done, vec![answered("task-1", "Yes, merge it")]));
    let (status, _) = call(&mut ports, &["status"], "");
    assert!(status.text.contains("waiting on the owner (flagged)\n  (none)\n"), "{}", status.text);
}

#[test]
fn wait_and_unwait_are_refused_for_unknown_finished_or_not_waiting_tasks_and_malformed_words_get_usage() {
    let refused = |events: Vec<Event>, args: &[&str]| {
        let (reply, added) = call(&mut given(events, Vec::new()), args, "");
        assert_eq!((reply.code, added), (Code::Refused, Vec::new()), "{args:?}: {}", reply.text);
    };
    refused(Vec::new(), &["wait", "task-1", "--owner", "q"]);
    refused(finished_task(), &["wait", "task-1", "--owner", "q"]);
    refused(
        vec![opened("task-1"), Event::Deferred { task: id("task-1"), reason: id("later") }],
        &["wait", "task-1", "--owner", "q"],
    );
    refused(vec![opened("task-1")], &["unwait", "task-1", "a"]);
    refused(Vec::new(), &["unwait", "task-1", "a"]);
    let done_while_waiting = [linked_task(), vec![waiting("task-1", "q"), merged("7", 1), done("task-1")]].concat();
    refused(done_while_waiting, &["unwait", "task-1", "a"]);
    [&["wait", "task-1", "q"][..], &["wait", "task-1", "--owner"], &["unwait", "task-1"]].into_iter().for_each(
        |args| {
            let reply = Call::parse(&command(args), String::new).unwrap_err();
            assert!(reply.code == Code::Failed && reply.text.contains("usage:"), "{args:?}: {}", reply.text);
        },
    );
}

#[test]
fn a_waiting_task_is_not_overdue_and_its_deadlines_resume_from_the_answer() {
    // The fake clock stands at unix 2; the lease is due at unix 100.
    let history = vec![opened("task-1"), claimed("task-1"), leased_until("lease-1", "task-1", 100)];
    let mut ports = given(history, Vec::new());
    call(&mut ports, &["wait", "task-1", "--owner", "q"], "");
    ports.clock.pause(Span::of_seconds(498));
    let (status, _) = call(&mut ports, &["status"], "");
    assert!(status.text.contains("overdue leases (flagged)\n  (none)\n"), "waiting at unix 500: {}", status.text);
    let (reply, events) = call(&mut ports, &["unwait", "task-1", "a"], "");
    assert_eq!((reply.code, events), (Code::Done, vec![answered("task-1", "a")]));
    // It waited from unix 2 to unix 500: the deadline moves from 100 to 598.
    ports.clock.pause(Span::of_seconds(98));
    let (status, _) = call(&mut ports, &["status"], "");
    assert!(status.text.contains("overdue leases (flagged)\n  (none)\n"), "unix 598: {}", status.text);
    assert!(status.text.contains("lease-1 for task-1, deadline unix 598"), "{}", status.text);
    ports.clock.pause(Span::of_seconds(1));
    let (status, _) = call(&mut ports, &["status"], "");
    assert!(status.text.contains("lease-1 for task-1: unused, deadline unix 598\n"), "unix 599: {}", status.text);
}

#[test]
fn unwait_moves_the_started_deadline_too_and_leaves_other_tasks_alone() {
    let started =
        Event::Started { lease: id("lease-1"), agent: id(AGENT_ID), deadline: Some(Timestamp::from_unix_seconds(200)) };
    let history = vec![
        opened("task-1"),
        claimed("task-1"),
        leased_until("lease-1", "task-1", 100),
        used("lease-1"),
        started,
        opened("task-2"),
        claimed("task-2"),
        leased_until("lease-2", "task-2", 100),
        waiting("task-1", "q"),
    ];
    let mut ports = given(history, Vec::new());
    ports.clock.pause(Span::of_seconds(1000));
    call(&mut ports, &["unwait", "task-1", "a"], "");
    let (status, _) = call(&mut ports, &["status"], "");
    let flagged = "overdue leases (flagged)\n  lease-2 for task-2: unused, deadline unix 100\n";
    assert!(status.text.contains(flagged), "task-1 resumed at unix 1002, due 1201: {}", status.text);
    assert!(!status.text.contains("lease-1 for task-1:"), "{}", status.text);
    ports.clock.pause(Span::of_seconds(200));
    let (status, _) = call(&mut ports, &["status"], "");
    assert!(
        status.text.contains(&format!("lease-1 for task-1: agent {AGENT_ID}, deadline unix 1201")),
        "{}",
        status.text
    );
}

#[test]
fn waiting_records_fold_old_logs_and_round_trip_as_json() {
    let wait = Record { at: Timestamp::from_unix_seconds(5), event: waiting("task-1", "q") };
    let line = serde_json::to_string(&wait).unwrap();
    assert_eq!(line, r#"{"at":5,"event":"WaitingOnOwner","task":1,"question":"q"}"#);
    assert_eq!(serde_json::from_str::<Record>(&line).unwrap(), wait);
    let twice = vec![opened("task-1"), waiting("task-1", "a"), waiting("task-1", "b")];
    let (reply, _) = call(&mut given(twice, Vec::new()), &["status"], "");
    assert_eq!(reply.code, Code::Failed, "{}", reply.text);
    let unasked = vec![opened("task-1"), answered("task-1", "a")];
    let (reply, _) = call(&mut given(unasked, Vec::new()), &["status"], "");
    assert_eq!(reply.code, Code::Failed, "{}", reply.text);
}

fn reviewed(lease: &str, verdict: &str) -> Event {
    Event::Reviewed { lease: id(lease), verdict: id(verdict) }
}

/// task-1 linked at PINNED with an old work lease, then a used review lease-2 at `head` with `verdict`.
fn judged(head: &str, verdict: &str) -> Vec<Event> {
    [linked_task(), vec![review_leased("lease-2", head), used("lease-2"), reviewed("lease-2", verdict)]].concat()
}

fn idle_line(stalled: &str) -> String {
    format!("idle (flagged)\n  {IDLE}{stalled}\n")
}

fn quiet(events: Vec<Event>) {
    let text = status_of(events);
    assert!(text.contains("idle (flagged)\n  (none)\n"), "{text}");
}

fn flagged(events: Vec<Event>, stalled: &str) {
    let text = status_of(events);
    assert!(text.contains(&idle_line(stalled)), "expected {stalled:?}: {text}");
}

#[test]
fn only_a_lease_with_a_deadline_on_an_open_or_claimed_task_is_running_work() {
    flagged(vec![opened("task-1")], "task-1 (open)");
    quiet(Vec::new());
    quiet(vec![opened("task-1"), Event::Deferred { task: id("task-1"), reason: id("later") }]);
    quiet(finished_task());
    quiet(vec![opened("task-1"), claimed("task-1"), leased_until("lease-1", "task-1", 100)]);
    quiet(
        [vec![opened("task-1"), claimed("task-1"), leased_until("lease-1", "task-1", 100)], vec![used("lease-1")]]
            .concat(),
    );
    // A lease with no deadline is not running work, so the claimed task is stalled.
    flagged(claimed_task(), "task-1 (claimed)");
    flagged(bound(), "task-1 (claimed)");
    // A review lease is not work.
    let reviewing = [
        vec![opened("task-1"), claimed("task-1"), linked("task-1", "7"), opened("task-2")],
        vec![review_leased_until("lease-1", PINNED, 100), used("lease-1")],
    ]
    .concat();
    flagged(reviewing, "task-2 (open)");
}

#[test]
fn a_lease_of_a_linked_task_does_not_silence_the_alarm() {
    let leftover = [
        vec![opened("task-1"), claimed("task-1"), leased_until("lease-1", "task-1", 100), linked("task-1", "7")],
        vec![opened("task-2")],
    ]
    .concat();
    flagged(leftover.clone(), "task-1 (no verdict on head), task-2 (open)");
    // The same leftover lease and nothing else: still stalled, still flagged.
    flagged(leftover[..4].to_vec(), "task-1 (no verdict on head)");
    // Running work on the other task silences the alarm, as before.
    let working = [leftover, vec![claimed("task-2"), leased_until("lease-2", "task-2", 100)]].concat();
    quiet(working);
}

#[test]
fn a_linked_task_with_no_next_step_is_stalled_without_an_age_threshold() {
    flagged(linked_task(), "task-1 (no verdict on head)");
    // A verdict on an earlier head does not count for the pinned head.
    flagged(judged(MOVED, "pass"), "task-1 (no verdict on head)");
    flagged(judged(PINNED, "pass"), "task-1 (pass on head, PR not merged)");
    flagged(judged(PINNED, "fail"), "task-1 (fail on head, no live work lease)");
    // A review lease that is live is the next step, used or not.
    quiet([linked_task(), vec![review_leased_until("lease-2", PINNED, 100)]].concat());
    quiet([linked_task(), vec![review_leased("lease-2", PINNED), used("lease-2")]].concat());
    // Merged and awaiting done, or done: no stall.
    quiet([judged(PINNED, "pass"), vec![merged("7", 1)]].concat());
    quiet([judged(PINNED, "pass"), vec![merged("7", 1), done("task-1")]].concat());
    // A fail followed by a new work lease with a deadline is the fix under way; a leftover one is not.
    let fixing = [judged(PINNED, "fail"), vec![leased_until("lease-3", "task-1", 100)]].concat();
    quiet(fixing);
    let leftover = [
        vec![opened("task-1"), claimed("task-1"), leased_until("lease-1", "task-1", 100), linked("task-1", "7")],
        vec![review_leased("lease-2", PINNED), used("lease-2"), reviewed("lease-2", "fail")],
    ]
    .concat();
    flagged(leftover, "task-1 (fail on head, no live work lease)");
    // Retired tasks are not stalled.
    quiet([linked_task(), vec![Event::Retired { task: id("task-1"), reason: id("dropped") }]].concat());
}

#[test]
fn a_lead_who_claimed_a_task_itself_counts_as_stalled() {
    flagged(vec![opened("task-1"), self_claimed("task-1")], "task-1 (claimed)");
    quiet(vec![opened("task-1"), self_claimed("task-1"), leased_until("lease-1", "task-1", 100)]);
}

#[test]
fn a_task_waiting_on_the_owner_is_not_stalled_and_its_lease_is_not_work() {
    quiet(vec![opened("task-1"), waiting("task-1", "q")]);
    quiet([linked_task(), vec![waiting("task-1", "q")]].concat());
    let paused = vec![
        opened("task-1"),
        opened("task-2"),
        claimed("task-2"),
        leased_until("lease-1", "task-2", 100),
        waiting("task-2", "q"),
    ];
    flagged(paused, "task-1 (open)");
    let other = vec![
        opened("task-1"),
        opened("task-2"),
        claimed("task-2"),
        leased_until("lease-1", "task-2", 100),
        waiting("task-1", "q"),
    ];
    quiet(other);
}

#[test]
fn the_alarm_lists_every_stalled_task_by_name_in_task_order() {
    let history = vec![
        opened("task-1"),
        opened("task-2"),
        opened("task-3"),
        opened("task-4"),
        claimed("task-3"),
        self_claimed("task-4"),
        waiting("task-2", "q"),
    ];
    flagged(history, "task-1 (open), task-3 (claimed), task-4 (claimed)");
}

#[test]
fn waiting_tasks_show_their_age_in_a_flagged_section() {
    let mut ports = given(vec![opened("task-1")], Vec::new());
    call(&mut ports, &["wait", "task-1", "--owner", "Merge", "it?"], "");
    ports.clock.pause(Span::of_seconds(3600 + 59));
    let (status, _) = call(&mut ports, &["status"], "");
    let listed = "waiting on the owner (flagged)\n  task-1: Merge it? (waiting 60 min)\n";
    assert!(status.text.contains(listed), "{}", status.text);
    let (next, _) = call(&mut ports, &["unwait", "task-1", "yes"], "");
    assert_eq!(next.code, Code::Done);
    let (status, _) = call(&mut ports, &["status"], "");
    assert!(status.text.contains("waiting on the owner (flagged)\n  (none)\n"), "{}", status.text);
}

#[test]
fn the_idle_alarm_follows_the_clock_through_a_wait_and_its_answer() {
    let mut ports = given(vec![opened("task-1")], Vec::new());
    call(&mut ports, &["wait", "task-1", "--owner", "q"], "");
    ports.clock.pause(Span::of_seconds(3600));
    let (status, _) = call(&mut ports, &["status"], "");
    assert!(status.text.contains("idle (flagged)\n  (none)\n"), "{}", status.text);
    assert!(status.text.contains("task-1: q (waiting 60 min)"), "{}", status.text);
    call(&mut ports, &["unwait", "task-1", "go"], "");
    let (status, _) = call(&mut ports, &["status"], "");
    assert!(status.text.contains(&idle_line("task-1 (open)")), "{}", status.text);
    call(&mut ports, &["claim", "task-1", "--agent", "alice"], "");
    let (status, _) = call(&mut ports, &["status"], "");
    assert!(status.text.contains("idle (flagged)\n  (none)\n"), "{}", status.text);
}

/// A task linked to `pr` by the lead, with no lease.
fn linked_by_lead(task: &str, pr: &str) -> Vec<Event> {
    vec![opened(task), self_claimed(task), Event::Linked { task: id(task), pr: id(pr), pin: id(PINNED) }]
}

/// One review round on `task`'s PR: a lease, its use and the verdict.
fn review_round(lease: &str, task: &str, pr: &str, verdict: &str) -> Vec<Event> {
    vec![
        Event::ReviewLeased {
            lease: id(lease),
            task: id(task),
            agent: None,
            pr: id(pr),
            head: id(PINNED),
            deadline: None,
        },
        used(lease),
        Event::Reviewed { lease: id(lease), verdict: id(verdict) },
    ]
}

/// The recorded `gh run list` reply of a PR's branch, with `runs` rows.
fn ran(runs: usize) -> GhReplies {
    let rows: Vec<serde_json::Value> = (1..=runs).map(|row| serde_json::json!({ "databaseId": row })).collect();
    GhReplies { runs: serde_json::Value::Array(rows).to_string(), ..pull(PINNED, &[Outcome::Green]) }
}

#[test]
fn status_reports_rework_per_task_from_ci_runs_and_failed_review_rounds_for_merged_open_and_retired_tasks() {
    let history = [
        linked_by_lead("task-1", "7"),
        review_round("lease-1", "task-1", "7", "fail"),
        review_round("lease-2", "task-1", "7", "fail"),
        review_round("lease-3", "task-1", "7", "pass"),
        linked_by_lead("task-2", "9"),
        vec![merged("9", 2), done("task-2")],
        linked_by_lead("task-3", "21"),
        review_round("lease-4", "task-3", "21", "fail"),
        review_round("lease-5", "task-3", "21", "fail"),
        review_round("lease-6", "task-3", "21", "fail"),
        review_round("lease-7", "task-3", "21", "fail"),
        linked_by_lead("task-4", "30"),
        linked_by_lead("task-5", "31"),
        review_round("lease-8", "task-5", "31", "fail"),
        vec![Event::Retired { task: id("task-5"), reason: id("superseded") }],
    ]
    .concat();
    let pulls = vec![("7", ran(3)), ("21", ran(1)), ("30", ran(1)), ("31", ran(5))];
    let (reply, added) = call(&mut given(history, pulls), &["status"], "");
    assert_eq!(reply.code, Code::Done, "{}", reply.text);
    assert!(added.iter().all(|event| matches!(event, Event::Falsified { .. })), "{added:?}");
    let expected = "rework per task (report only; not part of P6)\n\
        \x20 task-1 (#7, linked): failed review rounds 2, CI runs 3\n\
        \x20 task-2 (#9, done): failed review rounds 0, CI runs 2\n\
        \x20 task-3 (#21, linked): failed review rounds 4, CI runs 1\n\
        \x20 task-5 (#31, retired): failed review rounds 1, CI runs 5\n\
        rework total: tasks with 2 or more failed review rounds: 2 of 5 tasks with a PR; \
        worst: task-3 (failed review rounds 4, CI runs 1)\n";
    assert!(reply.text.contains(expected), "{}", reply.text);
    assert!(reply.text.contains("P6 CI runs per merged PR: 2 runs over 1 merged PRs; median 2\n"), "{}", reply.text);
}

#[test]
fn status_takes_a_merged_prs_ci_runs_from_the_log_and_never_reads_github_for_it() {
    let history = [linked_by_lead("task-2", "9"), vec![merged("9", 4), done("task-2")]].concat();
    let withheld = status_of(history.clone());
    let line = "  task-2 (#9, done): failed review rounds 0, CI runs 4\n";
    assert!(withheld.contains(line), "{withheld}");
    let (reply, _) = call(&mut given(history, vec![("9", ran(1))]), &["status"], "");
    assert!(reply.text.contains(line), "{}", reply.text);
}

#[test]
fn status_reads_an_open_prs_ci_runs_live_and_names_them_unavailable_when_github_fails() {
    let history =
        [linked_by_lead("task-1", "7"), linked_by_lead("task-2", "9"), vec![merged("9", 2), done("task-2")]].concat();
    let status = status_of(history);
    assert!(
        status.contains(
            "  task-1 (#7, linked): failed review rounds 0, CI runs unavailable (no recorded reply for #7)\n"
        ),
        "{status}"
    );
    assert!(status.contains("  task-2 (#9, done): failed review rounds 0, CI runs 2\n"), "{status}");
}

#[test]
fn status_reports_rework_as_none_without_a_linked_pr_and_names_ci_runs_it_could_not_read() {
    let status = status_of(vec![opened("task-1")]);
    let none = "rework per task (report only; not part of P6)\n  (none)\n\
        rework total: tasks with 2 or more failed review rounds: 0 of 0 tasks with a PR; worst: none\n";
    assert!(status.contains(none), "{status}");
    let history = [linked_by_lead("task-1", "7"), review_round("lease-1", "task-1", "7", "fail")].concat();
    let status = status_of(history);
    let unread = "  task-1 (#7, linked): failed review rounds 1, CI runs unavailable (no recorded reply for #7)\n";
    assert!(status.contains(unread), "{status}");
    assert!(status.contains("worst: task-1 (failed review rounds 1, CI runs unavailable)"), "{status}");
}

#[test]
fn brief_of_an_unused_live_lease_prints_the_brief_its_claim_printed_and_records_nothing() {
    let mut ports = given(vec![opened("task-1")], Vec::new());
    let (claim, _) = call(&mut ports, &["claim", "task-1", "--agent", "alice", "--deadline", "10"], "");
    ports.clock = FakeClock::at(Timestamp::from_unix_seconds(500));
    let (reply, events) = call(&mut ports, &["brief", "lease-1"], "");
    assert_eq!((reply, events), (claim, Vec::new()));
    let used = Event::LeaseUsed { lease: id("lease-1") };
    ports.log.records.push(Record { at: Timestamp::from_unix_seconds(3), event: used });
    let (fresh, _) = call(&mut ports, &["claim", "task-1", "--agent", "bob"], "");
    let (reply, events) = call(&mut ports, &["brief", "lease-2"], "");
    assert_eq!((reply.code, events), (Code::Done, Vec::new()));
    assert_eq!(reply.text, fresh.text);
    assert!(reply.text.contains("agent: bob\n") && reply.text.contains("lease: lease-2 "), "{}", reply.text);
}

#[test]
fn brief_of_an_unused_review_lease_prints_the_review_brief_its_review_printed() {
    let history = [linked_task(), vec![used("lease-1")]].concat();
    let mut ports = given(history, vec![("7", pull(MOVED, &[]))]);
    let (review, _) = call(&mut ports, &["review", "task-1", "--agent", "rev"], "");
    let (reply, events) = call(&mut ports, &["brief", "lease-2"], "");
    assert_eq!((reply, events), (review, Vec::new()));
    let (refused, _) = call(&mut ports, &["review", "task-1", "--agent", "rev"], "");
    assert!(refused.text.contains("meta brief lease-2 prints its brief again"), "{}", refused.text);
}

#[test]
fn brief_of_a_lease_recorded_without_an_agent_names_the_task_agent_or_says_none_was_recorded() {
    let (work, _) = call(&mut given(claimed_task(), Vec::new()), &["brief", "lease-1"], "");
    assert!(work.text.contains("agent: alice\n") && work.text.contains("deadline: none\n"), "{}", work.text);
    let history = [linked_task(), vec![used("lease-1"), review_leased("lease-2", MOVED)]].concat();
    let (review, _) = call(&mut given(history, Vec::new()), &["brief", "lease-2"], "");
    assert!(review.text.contains("agent: not recorded\n") && review.text.contains(MOVED), "{}", review.text);
}

#[test]
fn brief_of_a_used_unknown_or_dead_lease_is_refused_with_the_reason_and_records_nothing() {
    let spent = [claimed_task(), vec![used("lease-1")]].concat();
    let cases = [
        (spent, "lease-1", "already used"),
        (claimed_task(), "lease-9", "lease-9 is unknown"),
        (finished_task(), "lease-1", "not live"),
        (
            [claimed_task(), vec![Event::Retired { task: id("task-1"), reason: id("not wanted") }]].concat(),
            "lease-1",
            "not live",
        ),
    ];
    cases.into_iter().for_each(|(events, lease, why)| {
        let (reply, added) = call(&mut given(events, Vec::new()), &["brief", lease], "");
        assert_eq!((reply.code, added), (Code::Refused, Vec::new()));
        assert!(reply.text.contains(why), "{}", reply.text);
    });
    let reply = Call::parse(&command(&["brief", "task-1"]), String::new).unwrap_err();
    assert!(reply.code == Code::Failed && reply.text.contains("meta brief <lease>"), "{}", reply.text);
}

/// A side file line as a branch build writes it.
fn side_line(at: u64, kind: &str, pr: Option<&str>, reason: &str) -> String {
    let pr = pr.map_or_else(String::new, |pr| format!(",\"pr\":{pr}"));
    format!("{{\"at\":{at},\"kind\":\"{kind}\",\"reason\":\"{reason}\"{pr}}}\n")
}

const IMPORT: [&str; 2] = ["escape", "--import"];

#[test]
fn import_moves_side_file_escapes_into_the_log_with_ids_keeps_unreadable_lines_and_status_shows_them() {
    let scratch = Scratch::new("import");
    let mut ports = on_marked_jsonl(&scratch, Build::commit(id(MAIN_BUILD)));
    let (first, second) = (side_line(3, "gap", None, "from a branch"), side_line(4, "gap", None, "again"));
    std::fs::write(scratch.join(SIDE_FILE), [first.as_str(), "not a record\n", second.as_str()].concat()).unwrap();
    let reply = answer(parsed(&IMPORT), &mut ports);
    assert_eq!(reply.code, Code::Done, "{}", reply.text);
    let said = [
        "escape-1 imported: [gap] from a branch (from an unverified build)",
        "escape-2 imported: [gap] again (from an unverified build)",
        "1 unreadable lines kept in the side file",
    ];
    assert!(said.iter().all(|part| reply.text.contains(part)), "{}", reply.text);
    assert_eq!(std::fs::read_to_string(scratch.join(SIDE_FILE)).unwrap(), "not a record\n");
    let log = std::fs::read_to_string(scratch.join("log.jsonl")).unwrap();
    assert_eq!(log.bytes().filter(|byte| *byte == b'\n').count(), 3, "{log}");
    assert!(log.contains("\"imported\":{\"line\":") && log.contains("\"written\":3"), "{log}");
    let status = answer(parsed(&["status"]), &mut ports);
    let shown = [
        "escape-1 [gap] from a branch (imported from an unverified build)\n",
        "escape-2 [gap] again (imported from an unverified build)\n",
        "open escapes: 2\n",
        "1 unreadable lines",
    ];
    assert!(shown.iter().all(|part| status.text.contains(part)), "{}", status.text);
    assert!(!status.text.contains("unverified [gap]"), "{}", status.text);
    let resolved = answer(parsed(&["resolve", "escape-1", "--historical", "fixed", "upstream"]), &mut ports);
    assert_eq!(resolved.code, Code::Done, "{}", resolved.text);
    let again = answer(parsed(&["status"]), &mut ports);
    assert!(again.text.contains("open escapes: 1\n"), "{}", again.text);
}

#[test]
fn an_imported_escape_naming_a_pr_counts_for_p1_like_any_other_escape() {
    let scratch = Scratch::new("import-p1");
    let mut ports = on_marked_jsonl(&scratch, Build::commit(id(MAIN_BUILD)));
    let outside =
        vec![opened("task-2"), claimed("task-2"), linked("task-2", "8"), merged_outside("8", 1), done("task-2")];
    let history = [vec![Event::RealLog], finished_task(), outside].concat();
    let text: String = history
        .into_iter()
        .map(|event| serde_json::to_string(&Record { at: Timestamp::from_unix_seconds(1), event }).unwrap() + "\n")
        .collect();
    std::fs::write(scratch.join("log.jsonl"), text).unwrap();
    ports.github.listed = Some(Ok(listed(&[7, 8])));
    let before = answer(parsed(&["status"]), &mut ports).text;
    assert!(before.contains("P1 The loop is used: FALSIFIED; observed unrecorded bypasses 1;"), "{before}");
    std::fs::write(scratch.join(SIDE_FILE), side_line(3, "outside-merge", Some("8"), "merged by hand")).unwrap();
    assert_eq!(answer(parsed(&IMPORT), &mut ports).code, Code::Done);
    let after = answer(parsed(&["status"]), &mut ports).text;
    assert!(after.contains("P1 merged outside meta: 1 (0 missing from the log)"), "{after}");
    assert!(after.contains("UNDECIDED; observed unrecorded bypasses unavailable"), "{after}");
}

#[test]
fn import_is_refused_from_a_build_not_on_main_and_changes_neither_file() {
    unverified_builds().into_iter().for_each(|(build, why)| {
        let scratch = Scratch::new("import-refused");
        let mut ports = on_marked_jsonl(&scratch, build);
        let side = side_line(3, "gap", None, "from a branch");
        std::fs::write(scratch.join(SIDE_FILE), &side).unwrap();
        let reply = answer(parsed(&IMPORT), &mut ports);
        assert_eq!(reply.code, Code::Refused, "{}", reply.text);
        assert!(reply.text.contains(why), "{}", reply.text);
        assert_eq!(std::fs::read_to_string(scratch.join(SIDE_FILE)).unwrap(), side);
        assert_eq!(std::fs::read_to_string(scratch.join("log.jsonl")).unwrap(), MARKER_LINE);
    });
}

#[test]
fn a_crash_between_the_append_and_the_side_file_rewrite_imports_nothing_twice_and_loses_nothing() {
    let scratch = Scratch::new("import-crash");
    let mut ports = on_marked_jsonl(&scratch, Build::commit(id(MAIN_BUILD)));
    let side =
        [side_line(3, "gap", None, "one"), "not a record\n".to_owned(), side_line(4, "gap", None, "two")].concat();
    std::fs::write(scratch.join(SIDE_FILE), &side).unwrap();
    std::fs::create_dir(scratch.join("escapes.unverified.jsonl.importing")).unwrap();
    let cut = answer(parsed(&IMPORT), &mut ports);
    assert_eq!(cut.code, Code::Failed, "{}", cut.text);
    assert_eq!(std::fs::read_to_string(scratch.join(SIDE_FILE)).unwrap(), side);
    let log = std::fs::read_to_string(scratch.join("log.jsonl")).unwrap();
    assert_eq!(log.bytes().filter(|byte| *byte == b'\n').count(), 3, "{log}");
    std::fs::remove_dir(scratch.join("escapes.unverified.jsonl.importing")).unwrap();
    let retry = answer(parsed(&IMPORT), &mut ports);
    assert_eq!(retry.code, Code::Done, "{}", retry.text);
    assert!(retry.text.contains("2 already in the log from an earlier import"), "{}", retry.text);
    assert!(!retry.text.contains("imported:"), "{}", retry.text);
    assert_eq!(std::fs::read_to_string(scratch.join("log.jsonl")).unwrap(), log);
    assert_eq!(std::fs::read_to_string(scratch.join(SIDE_FILE)).unwrap(), "not a record\n");
    let last = answer(parsed(&IMPORT), &mut ports);
    assert!(last.text.contains("1 unreadable lines kept"), "{}", last.text);
    assert_eq!(std::fs::read_to_string(scratch.join("log.jsonl")).unwrap(), log);
}

#[test]
fn import_with_nothing_in_the_side_file_says_so_and_a_scratch_log_takes_it_from_any_build() {
    let scratch = Scratch::new("import-empty");
    let mut ports = on_marked_jsonl(&scratch, Build::commit(id(MAIN_BUILD)));
    let reply = answer(parsed(&IMPORT), &mut ports);
    assert_eq!((reply.code, reply.text.as_str()), (Code::Done, "the side file holds no escapes to import\n"));
    assert!(!scratch.join(SIDE_FILE).exists());
    let mut memory = Ports { build: Build::commit(id(BRANCH_BUILD)), ..given(Vec::new(), Vec::new()) };
    memory.log.set_aside = vec![serde_json::from_str(&side_line(3, "gap", None, "scratch")).unwrap(), None];
    let (reply, events) = call(&mut memory, &IMPORT, "");
    assert_eq!(reply.code, Code::Done, "{}", reply.text);
    assert_eq!((events.len(), memory.log.set_aside.len()), (1, 1));
    assert!(matches!(&events[0], Event::Escaped { imported: Some(_), .. }), "{events:?}");
}

#[test]
fn import_into_a_log_that_does_not_parse_fails_and_keeps_the_side_file() {
    let scratch = Scratch::new("import-garbled");
    let mut ports = on_marked_jsonl(&scratch, Build::commit(id(MAIN_BUILD)));
    std::fs::write(scratch.join("log.jsonl"), format!("{MARKER_LINE}garbled\n")).unwrap();
    let side = side_line(3, "gap", None, "one");
    std::fs::write(scratch.join(SIDE_FILE), &side).unwrap();
    let reply = answer(parsed(&IMPORT), &mut ports);
    assert_eq!(reply.code, Code::Failed, "{}", reply.text);
    assert_eq!(std::fs::read_to_string(scratch.join(SIDE_FILE)).unwrap(), side);
}
