use meta_cli::{Answer, Call, Code, answer};
use meta_core::{Attempt, Check, Event, Outcome, PrNumber, Pull, Record, RunCount, Sha, Timestamp};
use meta_ports::Ports;
use meta_ports::memory::{FixedClock, MemoryLog, RecordedGit, RecordedGitHub};
use std::str::FromStr;

const RULES: &str = include_str!("../../../self/rules.toml");
const ORIGIN: &str = "https://github.com/Studio-Terza/meta-dev.git";
const PINNED: &str = "aaaaaaa";
const MOVED: &str = "bbbbbbb";
const REQUIRED: &str = "gates";
const DENY: &str = r#""permissionDecision":"deny""#;
const ALLOW: &str = "{}\n";
const GET_LEASE: &str = "meta claim <task> --agent <name>";

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
    Event::Merged { pr: id(pr), head: id(PINNED), runs: RunCount::of(runs) }
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

fn pull(head: &str, outcomes: &[Outcome]) -> Pull {
    Pull {
        head: id(head),
        required: vec![id(REQUIRED)],
        reported: outcomes.iter().map(|outcome| Check { name: id(REQUIRED), outcome: *outcome }).collect(),
        runs: RunCount::of(3),
    }
}

fn given(events: Vec<Event>, pulls: Vec<(&str, Pull)>) -> Memory {
    let at = Timestamp::from_unix_seconds(1);
    Ports {
        log: MemoryLog { records: events.into_iter().map(|event| Record { at, event }).collect() },
        git: RecordedGit { origin: id(ORIGIN), rules: RULES.to_owned() },
        github: RecordedGitHub {
            pulls: pulls.into_iter().map(|(pr, pull)| (id::<PrNumber>(pr), pull)).collect(),
            merged: Vec::new(),
        },
        clock: FixedClock(Timestamp::from_unix_seconds(2)),
    }
}

fn call(ports: &mut Memory, args: &str, stdin: &str) -> (Answer, Vec<Event>) {
    let before = ports.log.records.len();
    let words: Vec<String> = args.split_whitespace().map(str::to_owned).collect();
    let reply = answer(Call::parse(&words, || stdin.to_owned()).unwrap(), ports);
    let added = ports.log.records[before..].iter().map(|record| record.event.clone()).collect();
    (reply, added)
}

fn hook(events: Vec<Event>, input: &str) -> (Answer, Vec<Event>) {
    call(&mut given(events, Vec::new()), "hook pre-tool-use", input)
}

fn agent(prompt: &str) -> String {
    serde_json::json!({ "tool_name": "Agent", "tool_input": { "prompt": prompt } }).to_string()
}

fn bash(command: &str) -> String {
    serde_json::json!({ "tool_name": "Bash", "tool_input": { "command": command } }).to_string()
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
    assert_eq!(events, vec![claimed("task-1"), leased("lease-1", "task-1")]);
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
        (claimed_task(), "task-1", "already claimed"),
        (finished, "task-1", "is done"),
        (Vec::new(), "task-9", "unknown"),
    ];
    cases.into_iter().for_each(|(events, task, reason)| {
        let (reply, added) = call(&mut given(events, Vec::new()), &format!("claim {task} --agent bob"), "");
        assert_eq!((reply.code, added), (Code::Refused, Vec::new()));
        assert!(reply.text.contains(reason), "{}", reply.text);
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
    let mut ports = given(linked_task(), vec![("7", pull(PINNED, &[Outcome::Pending, Outcome::Green]))]);
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
        ("7", pull(MOVED, &[Outcome::Green]), "head moved", "meta link task-1 7"),
        ("7", Pull { required: Vec::new(), ..pull(PINNED, &[Outcome::Green]) }, "no required checks", "missing"),
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
fn escape_records_escaped_and_prints_open_count_and_repeats_of_the_reason() {
    let mut ports = given(vec![escaped("gh is down"), escaped("hook too strict")], Vec::new());
    let (reply, events) = call(&mut ports, "escape gh is down", "");
    assert_eq!(events, vec![escaped("gh is down")]);
    assert!(reply.text.contains("open escapes: 3") && reply.text.contains("this reason repeated: 1"), "{}", reply.text);
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
        vec![escaped("gh is down"), escaped("gh is down"), escaped("hook too strict")],
        vec![Event::Denied { attempt: Attempt::AgentStart }, Event::Denied { attempt: Attempt::Merge }],
    ]
    .concat();
    let (reply, events) = call(&mut given(history, Vec::new()), "status", "");
    assert_eq!(events, Vec::new());
    let expected = [
        "task-1 claimed by alice",
        "task-3 open",
        "lease-1 for task-1",
        "open escapes: 3",
        "2x gh is down",
        "P1 agent starts through meta: 1 of 2",
        "P1 merges through meta: 1 of 2",
        "P2 open escapes: 3; repeats of an earlier reason: 1",
        "P6 CI runs per merged PR: 4 runs over 1 merged PRs",
    ];
    assert!(expected.iter().all(|line| reply.text.contains(line)), "{}", reply.text);
    assert!(!reply.text.contains("task-2") && !reply.text.contains("lease-2"), "{}", reply.text);
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
        (claimed_task(), "Work on task-1.", "no lease"),
        (claimed_task(), "Work under lease-9.", "lease-9 is unknown"),
        (used, "Work under lease-1.", "already used"),
        (dead, "Work under lease-1.", "not live"),
    ];
    cases.into_iter().for_each(|(events, prompt, why)| {
        let (reply, added) = hook(events, &agent(prompt));
        assert_eq!(added, vec![Event::Denied { attempt: Attempt::AgentStart }]);
        assert!([DENY, why, GET_LEASE].iter().all(|part| reply.text.contains(part)), "{}", reply.text);
    });
}

#[test]
fn hook_denies_bash_that_merges_or_pushes_to_main_outside_meta_merge() {
    let bypasses = [
        "gh pr merge 7 --squash",
        "cd repo && gh pr merge 7",
        "gh api -X PUT repos/o/r/pulls/7/merge",
        "gh api graphql -f query='mutation { mergePullRequest(input: {}) { clientMutationId } }'",
        "git push origin main",
        "git push origin HEAD:refs/heads/main",
        "git -C repo push --force origin +main",
        "git push --all origin",
    ];
    bypasses.into_iter().for_each(|command| {
        let (reply, events) = hook(Vec::new(), &bash(command));
        assert_eq!(events, vec![Event::Denied { attempt: Attempt::Merge }], "{command}");
        assert!([DENY, "meta merge", "meta escape"].iter().all(|part| reply.text.contains(part)), "{}", reply.text);
    });
    let allowed =
        ["git push -u origin stage0", "gh pr view 7", "meta merge 7", "git commit -m 'merge main'", "git status"];
    allowed.into_iter().for_each(|command| {
        assert_eq!(hook(Vec::new(), &bash(command)), (Answer { text: ALLOW.to_owned(), code: Code::Done }, Vec::new()));
    });
}

#[test]
fn hook_denies_input_that_cannot_be_parsed() {
    let inputs = ["", "not json", "{}", r#"{"tool_name":"Bash","tool_input":{}}"#, r#"{"tool_name":"Agent"}"#];
    inputs.into_iter().for_each(|input| {
        let (reply, events) = hook(claimed_task(), input);
        assert_eq!(events, Vec::new());
        assert!(reply.text.contains(DENY) && reply.text.contains("fail closed"), "{}", reply.text);
    });
}
