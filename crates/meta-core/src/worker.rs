use crate::brief::Reissue;
use crate::event::Event;
use crate::ids::{
    AgentId, Binary, ClaudeVersion, ExitStatus, LeaseId, Malformed, Model, Origin, PrNumber, ProcessGroup, Reason, Sha,
    Timestamp, ToolRule, Transcript, Verdict,
};
use crate::lease::{Deadline, LeaseSlot, Purpose};
use crate::ledger::Ledger;
use crate::pull::Pull;
use crate::reply::{Accepted, Refusal};
use crate::rules::Rules;
use crate::spec::{SelfFile, Unreadable};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Headless workers' limits and tools from `self/workers.toml`, each with a source.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Workers {
    claude: Floor,
    model: Models,
    concurrency: Concurrency,
    review: Tools,
}

/// The oldest `claude` a worker may run on.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Floor {
    floor: ClaudeVersion,
    #[allow(dead_code)]
    source: Reason,
}

/// The model a session runs on. `--restricted` ignores user settings, so a session gets no other.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Models {
    review: Model,
    #[allow(dead_code)]
    source: Reason,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Concurrency {
    running: usize,
    #[allow(dead_code)]
    source: Reason,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Tools {
    allow: Vec<Tool>,
    deny: Denied,
}

/// The tools a session is denied, which always hold `Denied::SPAWNING`.
#[derive(Debug, Deserialize)]
#[serde(try_from = "Vec<Tool>")]
struct Denied(Vec<Tool>);

impl Denied {
    /// A session must never start another agent (`docs/prereg.md`, "Headless workers").
    const SPAWNING: [&str; 2] = ["Agent", "Task"];
}

impl TryFrom<Vec<Tool>> for Denied {
    type Error = Malformed;

    fn try_from(tools: Vec<Tool>) -> Result<Self, Malformed> {
        let names: Vec<String> = tools.iter().map(|tool| tool.tool.to_string()).collect();
        let missing = Self::SPAWNING.iter().find(|tool| !names.iter().any(|name| name == *tool));
        match missing {
            Some(tool) => Err(Malformed::because(format!("the deny list lacks {tool}"))),
            None => Ok(Self(tools)),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Tool {
    tool: ToolRule,
    #[allow(dead_code)]
    source: Reason,
}

impl Workers {
    pub fn parse(text: &str) -> Result<Self, Unreadable> {
        SelfFile::Workers.parse(text)
    }

    /// Refused when `installed` is below the floor, for the flags that confine a session may be
    /// missing from it.
    pub fn admit(&self, installed: ClaudeVersion) -> Result<(), Refusal> {
        let floor = self.claude.floor;
        match installed < floor {
            true => Err(Refusal::because(format!(
                "claude {installed} is below the floor {floor} in self/workers.toml. Next: update claude, then start the worker again."
            ))),
            false => Ok(()),
        }
    }
}

impl Tools {
    fn rules(tools: &[Tool]) -> String {
        tools.iter().map(|tool| tool.tool.to_string()).collect::<Vec<_>>().join(",")
    }
}

/// The running session of a worker, as the spawn port reported it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerRun {
    pub group: ProcessGroup,
    pub transcript: Transcript,
    pub model: Model,
}

/// What the spawn port sees of a worker's session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Session {
    Running,
    /// The session is gone, with its exit status when it left one.
    Ended(Option<ExitStatus>),
}

/// How an agent is known to have ended. A subagent has one end only: `Stopped`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkerEnd {
    Exited(ExitStatus),
    /// The session is gone and left no exit status: something outside meta killed it.
    Vanished,
    /// `meta worker wait` killed it at its deadline.
    Deadline,
    /// `meta stop` killed the worker, or a TaskStop stopped the subagent.
    Stopped,
}

impl From<Option<ExitStatus>> for WorkerEnd {
    fn from(status: Option<ExitStatus>) -> Self {
        status.map_or(Self::Vanished, Self::Exited)
    }
}

impl fmt::Display for WorkerEnd {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exited(status) => write!(formatter, "exited with status {status}"),
            Self::Vanished => formatter.write_str("ended with no exit status"),
            Self::Deadline => formatter.write_str("was killed at its deadline"),
            Self::Stopped => formatter.write_str("was stopped by meta stop"),
        }
    }
}

impl WorkerEnd {
    /// Why this end leaves no verdict; only a session that exited cleanly gave one.
    pub fn no_verdict(self) -> Option<NoVerdict> {
        match self {
            Self::Exited(status) if status.is_success() => None,
            other => Some(NoVerdict(format!("the worker {other}"))),
        }
    }
}

/// The change as meta read it from GitHub, for the session's scratch directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewCopy {
    pub view: String,
    pub diff: String,
}

impl ReviewCopy {
    /// The file that holds `gh pr view`.
    pub const VIEW: &str = "pr-view.json";
    /// The file that holds `gh pr diff`.
    pub const DIFF: &str = "pr.diff";
}

/// What a review session must end with (`--json-schema`).
const VERDICT_SCHEMA: &str = r#"{"type":"object","properties":{"verdict":{"enum":["pass","fail"]},"findings":{"type":"array","items":{"type":"string"}}},"required":["verdict","findings"],"additionalProperties":false}"#;

/// Why a session left no verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoVerdict(String);

impl NoVerdict {
    pub fn because(why: impl ToString) -> Self {
        Self(why.to_string())
    }
}

impl fmt::Display for NoVerdict {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// The session's structured output, as `VERDICT_SCHEMA` shapes it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionVerdict {
    verdict: Verdict,
    findings: Vec<String>,
}

/// Any line of a transcript that is a JSON object naming its type.
#[derive(Deserialize)]
struct TranscriptEvent {
    #[serde(rename = "type")]
    kind: String,
}

/// The last event of a `stream-json` run (Claude Code docs, headless, structured output).
#[derive(Deserialize)]
struct SessionResult {
    subtype: String,
    is_error: bool,
    structured_output: Option<SessionVerdict>,
}

impl SessionVerdict {
    const RESULT: &str = "result";
    const SUCCESS: &str = "success";

    /// The verdict in the transcript's last result event. The transcript is the session's stdout
    /// alone, and a line that is not an event is stepped over. Anything but a successful result
    /// with a valid structured output is no verdict.
    pub fn from_transcript(lines: &[String]) -> Result<Self, NoVerdict> {
        let is_result = |line: &&String| {
            serde_json::from_str::<TranscriptEvent>(line).is_ok_and(|event| event.kind == Self::RESULT)
        };
        let line = lines.iter().rev().find(is_result);
        let line = line.ok_or_else(|| NoVerdict::because("the transcript holds no result"))?;
        let result: SessionResult = serde_json::from_str(line)
            .map_err(|error| NoVerdict(format!("the result is not a valid verdict: {error}")))?;
        match (result.is_error, result.subtype.as_str(), result.structured_output) {
            (false, Self::SUCCESS, Some(verdict)) => Ok(verdict),
            (false, Self::SUCCESS, None) => Err(NoVerdict::because("the result has no structured output")),
            (_, subtype, _) => Err(NoVerdict(format!("the session failed: {subtype}"))),
        }
    }

    pub fn verdict(&self) -> Verdict {
        self.verdict
    }
}

/// What a finished session leaves: its verdict and the PR as GitHub reports it now, or why none.
pub type Judged = Result<(SessionVerdict, Pull), NoVerdict>;

/// One headless session to start, and the supervisor that watches it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerPlan {
    lease: LeaseId,
    id: AgentId,
    prompt: String,
    model: Model,
    tools: String,
    denied: String,
    copy: ReviewCopy,
    repository: Origin,
    binary: Binary,
}

impl WorkerPlan {
    pub fn id(&self) -> &AgentId {
        &self.id
    }

    /// The model the session runs on.
    pub fn model(&self) -> &Model {
        &self.model
    }

    /// What meta fetched for the session to read.
    pub fn copy(&self) -> &ReviewCopy {
        &self.copy
    }

    /// The repository the supervisor asks GitHub about from the worker's directory, which is no
    /// clone (`GH_REPO`).
    pub fn repository(&self) -> &Origin {
        &self.repository
    }

    pub fn program(&self) -> &'static str {
        "claude"
    }

    /// The headless flags, each from the Claude Code docs (`docs/stage2c.md`). `--tools` is what
    /// makes the session's tools an allowlist; `--allowedTools` alone only skips prompts. A prompt
    /// after `--tools` or the other list flags would be read as one more list item, so it comes
    /// first.
    pub fn arguments(&self) -> Vec<String> {
        let tools = self.tools.as_str();
        let model = self.model.to_string();
        [
            "-p",
            self.prompt.as_str(),
            "--restricted",
            "--model",
            model.as_str(),
            "--tools",
            tools,
            "--allowedTools",
            tools,
            "--disallowedTools",
            self.denied.as_str(),
            "--json-schema",
            VERDICT_SCHEMA,
            "--no-session-persistence",
            "--permission-mode",
            "dontAsk",
            "--permission-prompts",
            "none",
            "--strict-mcp-config",
            "--output-format",
            "stream-json",
            "--verbose",
        ]
        .map(str::to_owned)
        .to_vec()
    }

    /// The supervisor's program: this same binary.
    pub fn binary(&self) -> &Binary {
        &self.binary
    }

    pub fn supervisor_arguments(&self) -> [String; 3] {
        ["worker".to_owned(), "wait".to_owned(), self.lease.to_string()]
    }
}

/// A lease the ledger lets a worker start on, and the deadline counted from this start.
#[derive(Debug)]
pub struct Starting {
    lease: LeaseId,
    reissue: Reissue,
    deadline: Timestamp,
    pr: PrNumber,
    head: Sha,
    running: Vec<LeaseId>,
}

/// A worker within the limits, not started yet.
#[derive(Debug)]
pub struct Planned {
    lease: LeaseId,
    plan: WorkerPlan,
    deadline: Timestamp,
}

impl Starting {
    /// The PR the worker will review, to fetch for it.
    pub fn pr(&self) -> PrNumber {
        self.pr
    }

    /// Refused when `workers`' limit of running workers is reached.
    pub fn within_limit(&self, workers: &Workers) -> Result<(), Refusal> {
        let running = &self.running;
        let cap = workers.concurrency.running;
        if running.len() >= cap {
            let holders: Vec<String> = running.iter().map(ToString::to_string).collect();
            return Err(Refusal::because(format!(
                "{} workers are running ({}) and {} is the limit in self/workers.toml. Next: wait for one to finish, or meta stop <lease>.",
                running.len(),
                holders.join(", "),
                cap
            )));
        }
        Ok(())
    }

    /// Refused unless `pull`, read after the copy, stands at the head the lease reviews: a copy
    /// fetched at any other head is not the change this lease judges.
    pub fn at_head(&self, pull: &Pull) -> Result<(), Refusal> {
        let Self { lease, pr, head, .. } = self;
        match pull.is_at(head) {
            true => Ok(()),
            false => Err(Refusal::because(format!(
                "{pr} is at head {}, but {lease} reviews head {head}. Next: meta review <task> --agent <name> issues a lease for the current head.",
                pull.head()
            ))),
        }
    }

    /// The plan of this worker, with the change fetched for it.
    pub fn plan(
        self,
        workers: &Workers,
        copy: ReviewCopy,
        origin: &Origin,
        binary: &Binary,
        rules: &Rules,
    ) -> Result<Planned, Refusal> {
        self.within_limit(workers)?;
        let Self { lease, reissue, deadline, pr, head, .. } = self;
        let id = AgentId::worker(lease);
        let prompt = reissue.worker_brief(&id, pr, &head, Deadline::of(Some(deadline)), rules);
        let plan = WorkerPlan {
            lease,
            id,
            prompt,
            model: workers.model.review.clone(),
            tools: Tools::rules(&workers.review.allow),
            denied: Tools::rules(&workers.review.deny.0),
            copy,
            repository: origin.clone(),
            binary: binary.clone(),
        };
        Ok(Planned { lease, plan, deadline })
    }
}

impl Planned {
    pub fn session(&self) -> &WorkerPlan {
        &self.plan
    }

    /// The lease is used and the worker recorded, now that its session runs.
    pub fn started(self, run: WorkerRun) -> Accepted {
        let Self { lease, plan, deadline } = self;
        let text = format!(
            "{lease}: worker {} started, model {}, process group {}, deadline {deadline}, transcript {}\n",
            plan.id, run.model, run.group, run.transcript
        );
        let events = vec![
            Event::LeaseUsed { lease },
            Event::Started { lease, agent: plan.id, deadline: Some(deadline), worker: Some(run) },
        ];
        Accepted::new(events, text)
    }
}

/// A worker whose session has not been recorded as ended.
#[derive(Debug, Clone)]
pub struct Worker {
    lease: LeaseId,
    agent: AgentId,
    run: WorkerRun,
    deadline: Deadline,
    pr: PrNumber,
    head: Sha,
}

/// What `meta worker wait` finds on its lease.
#[derive(Debug)]
pub enum Watch {
    Running(Worker),
    /// The end of the worker is already recorded.
    Over,
}

impl Worker {
    pub fn agent(&self) -> &AgentId {
        &self.agent
    }

    pub fn run(&self) -> &WorkerRun {
        &self.run
    }

    pub fn pr(&self) -> PrNumber {
        self.pr
    }

    /// The head the worker reviewed.
    pub fn head(&self) -> &Sha {
        &self.head
    }

    /// Whether the worker's deadline has passed at `now`. A task waiting on the owner does not
    /// hold it back: a running session never goes without a deadline.
    pub fn is_past_deadline(&self, now: Timestamp) -> bool {
        self.deadline.passed(now)
    }

    /// The record of its end.
    pub fn ended(self, end: WorkerEnd) -> Accepted {
        let Self { lease, agent, .. } = self;
        Accepted::new(vec![Event::Finished { lease, end: Some(end) }], format!("{lease}: worker {agent} {end}\n"))
    }

    /// The lease, for what a supervisor reports.
    pub fn lease(&self) -> LeaseId {
        self.lease
    }
}

impl Ledger {
    /// Allows a worker on `lease`: a live, unused review lease with a deadline. The deadline counts
    /// from `at`, as it does for an agent the hook starts.
    pub fn worker(&self, lease: LeaseId, at: Timestamp) -> Result<Starting, Refusal> {
        let reissue = self.reissue(lease)?;
        let slot = self.leases.get(&lease).map(LeaseSlot::lease);
        let unused = slot.ok_or_else(|| Refusal::because(format!("{lease} is unknown. Next: meta status.")))?;
        let Purpose::Review { pr, head } = unused.purpose().clone() else {
            return Err(Refusal::because(format!(
                "{lease} is a work lease; a worker runs only a review lease for now. Next: start the agent with its lease."
            )));
        };
        let deadline = unused.deadline_from(at);
        let time = deadline.time().ok_or_else(|| {
            Refusal::because(format!(
                "{lease} has no deadline, and a worker must have one. Next: start the reviewer with its lease."
            ))
        })?;
        let running = self.running_workers().into_iter().map(|worker| worker.lease).collect();
        Ok(Starting { lease, reissue, deadline: time, pr, head, running })
    }

    /// The workers whose end is not recorded, whatever became of their tasks.
    fn running_workers(&self) -> Vec<Worker> {
        self.leases.iter().filter_map(|(id, slot)| Self::running_worker(*id, slot)).collect()
    }

    fn running_worker(lease: LeaseId, slot: &LeaseSlot) -> Option<Worker> {
        let run = slot.running_worker()?.clone();
        let agent = slot.agent()?.clone();
        let Purpose::Review { pr, head } = slot.lease().purpose().clone() else { return None };
        Some(Worker { lease, agent, run, deadline: slot.deadline(), pr, head })
    }

    /// The record of `worker`'s end, with the verdict its session left on the head it judged. With
    /// none, the status report says so (`unjudged`) and this says why, for the supervisor's log.
    pub fn worker_ended(&self, worker: Worker, end: WorkerEnd, judged: Judged) -> Accepted {
        let lease = worker.lease;
        let (finished, line) = worker.ended(end).into_parts();
        let reviewed = judged.and_then(|(session, pull)| {
            let (events, text) =
                self.verdict_of(lease, session.verdict, &pull).map_err(NoVerdict::because)?.into_parts();
            Ok((events, text, session.findings))
        });
        match reviewed {
            Ok((mut events, text, findings)) => {
                events.extend(finished);
                let findings: String = findings.iter().map(|finding| format!("  {finding}\n")).collect();
                Accepted::new(events, format!("{line}{text}{findings}"))
            }
            Err(why) => Accepted::new(finished, format!("{line}{lease}: review finished with no verdict: {why}\n")),
        }
    }

    /// The worker on `lease`, for its supervisor.
    pub fn watch(&self, lease: LeaseId) -> Result<Watch, Refusal> {
        let slot = self.leases.get(&lease).ok_or_else(|| Refusal::because(format!("{lease} is unknown.")))?;
        match (Self::running_worker(lease, slot), slot.is_worker()) {
            (Some(worker), _) => Ok(Watch::Running(worker)),
            (None, true) => Ok(Watch::Over),
            (None, false) => Err(Refusal::because(format!("{lease} started no worker. Next: meta status."))),
        }
    }

    /// The workers `meta stop` ends: the one on `lease`, or every running one.
    pub fn stopping(&self, lease: Option<LeaseId>) -> Result<Vec<Worker>, Refusal> {
        match lease {
            None => Ok(self.running_workers()),
            Some(lease) => match self.watch(lease)? {
                Watch::Running(worker) => Ok(vec![worker]),
                Watch::Over => {
                    Err(Refusal::because(format!("the worker on {lease} has already ended. Next: meta status.")))
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ClaudeVersion, NoVerdict, SessionVerdict, VERDICT_SCHEMA, Verdict, WorkerEnd, Workers};

    const REPO: &str = include_str!("../../../self/workers.toml");

    #[test]
    fn the_repo_gives_reviewers_three_tools_on_opus_denies_spawning_and_runs_at_most_4_workers() {
        let workers = Workers::parse(REPO).unwrap();
        assert_eq!(workers.concurrency.running, 4);
        assert_eq!(workers.model.review.to_string(), "opus");
        assert_eq!(super::Tools::rules(&workers.review.allow), "Read,Grep,Glob");
        assert_eq!(super::Tools::rules(&workers.review.deny.0), "Agent,Task,Bash,mcp__*");
    }

    const CLAUDE: &str = "[claude]\nfloor = \"2.1.314\"\nsource = \"s\"\n";
    const MODEL: &str = "[model]\nreview = \"opus\"\nsource = \"s\"\n";
    const LIMIT: &str = "[concurrency]\nrunning = 4\nsource = \"s\"\n";

    /// A `[[review.deny]]` entry for each of `tools`.
    fn denying(tools: &[&str]) -> String {
        tools.iter().map(|tool| format!("[[review.deny]]\ntool = \"{tool}\"\nsource = \"s\"\n")).collect()
    }

    /// The review section with no allowed tools, denying what a session must not have.
    fn review() -> String {
        format!("[review]\nallow = []\n{}", denying(&["Agent", "Task"]))
    }

    #[test]
    fn the_floor_admits_its_own_version_and_newer_and_compares_numbers_not_text() {
        let workers = Workers::parse(REPO).unwrap();
        let admits = |version: &str| workers.admit(version.parse().unwrap()).is_ok();
        assert!(admits("2.1.314") && admits("2.1.1000") && admits("2.2.0") && admits("10.0.0"));
        assert!(!admits("2.1.313") && !admits("2.0.999") && !admits("1.9.999"));
        let refusal = workers.admit("2.1.99".parse().unwrap()).unwrap_err().to_string();
        assert!(refusal.contains("claude 2.1.99 is below the floor 2.1.314 in self/workers.toml"), "{refusal}");
    }

    #[test]
    fn a_version_reply_parses_by_its_first_word_and_anything_else_is_malformed() {
        assert_eq!(ClaudeVersion::from_reply("2.1.314 (Claude Code)\n"), "2.1.314".parse());
        let bad = ["", "\n", "claude 2.1.314", "2.1", "2.1.3.4", "2.1.x", "2.1.+3", "2.1.03", "-2.1.3"];
        assert!(bad.iter().all(|reply| ClaudeVersion::from_reply(reply).is_err()), "{bad:?}");
    }

    #[test]
    fn workers_without_a_floor_do_not_parse() {
        let rest = format!("{MODEL}{LIMIT}{}", review());
        assert!(Workers::parse(&rest).is_err());
        assert!(Workers::parse(&format!("{CLAUDE}{rest}")).is_ok());
        assert!(Workers::parse(&format!("[claude]\nfloor = \"new\"\nsource = \"s\"\n{rest}")).is_err());
        assert!(Workers::parse(&format!("[claude]\nfloor = \"2.1.314\"\n{rest}")).is_err());
    }

    #[test]
    fn workers_without_a_model_with_an_empty_one_or_with_no_source_do_not_parse() {
        let with = |model: &str| Workers::parse(&format!("{CLAUDE}{model}{LIMIT}{}", review()));
        assert_eq!(with(MODEL).unwrap().model.review.to_string(), "opus");
        for model in ["", "[model]\nsource = \"s\"\n", "[model]\nreview = \"opus\"\n"] {
            assert!(with(model).is_err(), "{model}");
        }
        for model in ["[model]\nreview = \"\"\nsource = \"s\"\n", "[model]\nreview = \" \"\nsource = \"s\"\n"] {
            let why = with(model).unwrap_err().to_string();
            assert!(why.contains("model is empty"), "{why}");
        }
    }

    #[test]
    fn a_deny_list_that_lacks_agent_or_task_does_not_parse() {
        let with = |deny: String| Workers::parse(&format!("{CLAUDE}{MODEL}{LIMIT}[review]\nallow = []\n{deny}"));
        let why = |tools: &[&str]| with(denying(tools)).unwrap_err().to_string();
        assert!(with("deny = []\n".to_owned()).unwrap_err().to_string().contains("the deny list lacks Agent"));
        assert!(why(&["Agent"]).contains("the deny list lacks Task"));
        assert!(why(&["Task", "Bash"]).contains("the deny list lacks Agent"));
        assert!(why(&["Agent(reviewer)", "agent", "Task"]).contains("the deny list lacks Agent"));
        assert!(with(denying(&["Bash", "Task", "Agent"])).is_ok());
    }

    /// A result event as a real run wrote it (the probe of these flags), cut to what is read.
    fn result(subtype: &str, is_error: bool, output: &str) -> String {
        format!(
            r#"{{"type":"result","subtype":"{subtype}","is_error":{is_error},"num_turns":5,"result":"{{}}","structured_output":{output}}}"#
        )
    }

    fn output(verdict: &str) -> String {
        format!(r#"{{"verdict":"{verdict}","findings":["a.txt:1 reads hello"]}}"#)
    }

    fn read(lines: &[&str]) -> Result<SessionVerdict, NoVerdict> {
        SessionVerdict::from_transcript(&lines.iter().map(ToString::to_string).collect::<Vec<_>>())
    }

    #[test]
    fn the_verdict_is_the_last_result_event_and_a_line_that_is_no_event_does_not_matter() {
        let (pass, fail) = (result("success", false, &output("pass")), result("success", false, &output("fail")));
        let verdict = read(&[r#"{"type":"system","subtype":"init"}"#, "Waiting for background work", &pass, "bye"]);
        assert_eq!(verdict.unwrap().verdict(), Verdict::Pass);
        assert_eq!(read(&[&pass, &fail]).unwrap().verdict(), Verdict::Fail);
    }

    #[test]
    fn anything_but_a_successful_result_with_a_valid_structured_output_is_no_verdict() {
        let pass = output("pass");
        let cases: [(String, &str); 8] = [
            ("{\"type\":\"assistant\"}".to_owned(), "holds no result"),
            ("{\"type\":\"res".to_owned(), "holds no result"),
            (result("success", true, &pass), "the session failed: success"),
            (result("error_max_turns", false, &pass), "the session failed: error_max_turns"),
            (r#"{"type":"result","subtype":"success","is_error":false}"#.to_owned(), "no structured output"),
            (result("success", false, &output("maybe")), "not a valid verdict"),
            (result("success", false, r#"{"verdict":"pass"}"#), "not a valid verdict"),
            (result("success", false, r#"{"verdict":"pass","findings":[],"extra":1}"#), "not a valid verdict"),
        ];
        for (line, said) in cases {
            let why = read(&[&line]).unwrap_err().to_string();
            assert!(why.contains(said), "{line}: {why}");
        }
    }

    #[test]
    fn only_a_clean_exit_can_leave_a_verdict() {
        use crate::ids::ExitStatus;
        assert_eq!(WorkerEnd::Exited(ExitStatus::of(0)).no_verdict(), None);
        let ends = [WorkerEnd::Exited(ExitStatus::of(1)), WorkerEnd::Vanished, WorkerEnd::Deadline, WorkerEnd::Stopped];
        assert!(ends.iter().all(|end| end.no_verdict().is_some()));
    }

    #[test]
    fn the_verdict_schema_is_json_that_names_the_fields_a_verdict_parses_from() {
        let schema: serde_json::Value = serde_json::from_str(VERDICT_SCHEMA).unwrap();
        assert_eq!(schema["required"], serde_json::json!(["verdict", "findings"]));
        assert_eq!(schema["properties"]["verdict"]["enum"], serde_json::json!(["pass", "fail"]));
    }

    #[test]
    fn a_tool_or_limit_with_no_source_does_not_parse() {
        let deny = denying(&["Agent", "Task"]);
        let allow = |source: &str| format!("{CLAUDE}{MODEL}{LIMIT}[[review.allow]]\ntool = \"Read\"\n{source}{deny}");
        assert!(Workers::parse(&allow("source = \"s\"\n")).is_ok());
        let limit = format!("{CLAUDE}{MODEL}[concurrency]\nrunning = 4\n{}", review());
        let tool = allow("");
        let empty = allow("source = \" \"\n");
        assert!([limit, tool, empty].iter().all(|text| Workers::parse(text).is_err()));
    }
}
