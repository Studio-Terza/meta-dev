use crate::brief::Reissue;
use crate::event::Event;
use crate::ids::{AgentId, Binary, ExitStatus, LeaseId, Origin, ProcessGroup, Reason, Timestamp, ToolRule, Transcript};
use crate::lease::{Deadline, LeaseSlot, Purpose};
use crate::ledger::Ledger;
use crate::reply::{Accepted, Refusal};
use crate::rules::Rules;
use crate::spec::{SelfFile, Unreadable};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Headless workers' limits and tools from `self/workers.toml`, each with a source.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Workers {
    concurrency: Concurrency,
    review: Tools,
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
    deny: Vec<Tool>,
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
}

/// What the spawn port sees of a worker's session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Session {
    Running,
    /// The session is gone, with its exit status when it left one.
    Ended(Option<ExitStatus>),
}

/// How a worker ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkerEnd {
    Exited(ExitStatus),
    /// The session is gone and left no exit status: something outside meta killed it.
    Vanished,
    /// `meta worker wait` killed it at its deadline.
    Deadline,
    /// `meta stop` killed it.
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

/// One headless session to start, and the supervisor that watches it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerPlan {
    lease: LeaseId,
    id: AgentId,
    prompt: String,
    tools: String,
    denied: String,
    repository: Origin,
    binary: Binary,
}

impl WorkerPlan {
    pub fn id(&self) -> &AgentId {
        &self.id
    }

    /// The repository `gh` reads from the worker's scratch directory, which is no clone (`GH_REPO`).
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
        [
            "-p",
            self.prompt.as_str(),
            "--tools",
            tools,
            "--allowedTools",
            tools,
            "--disallowedTools",
            self.denied.as_str(),
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
    /// The plan of this worker; refused when `workers`' limit of running workers is reached.
    pub fn plan(self, workers: &Workers, origin: &Origin, binary: &Binary, rules: &Rules) -> Result<Planned, Refusal> {
        let Self { lease, reissue, deadline, running } = self;
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
        let (_, prompt) = reissue.brief(origin, binary, rules).into_parts();
        let plan = WorkerPlan {
            lease,
            id: AgentId::worker(lease),
            prompt,
            tools: Tools::rules(&workers.review.allow),
            denied: Tools::rules(&workers.review.deny),
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
            "{lease}: worker {} started, process group {}, deadline {deadline}, transcript {}\n",
            plan.id, run.group, run.transcript
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
}

impl Ledger {
    /// Allows a worker on `lease`: a live, unused review lease with a deadline. The deadline counts
    /// from `at`, as it does for an agent the hook starts.
    pub fn worker(&self, lease: LeaseId, at: Timestamp) -> Result<Starting, Refusal> {
        let reissue = self.reissue(lease)?;
        let slot = self.leases.get(&lease).map(LeaseSlot::lease);
        let unused = slot.ok_or_else(|| Refusal::because(format!("{lease} is unknown. Next: meta status.")))?;
        if *unused.purpose() == Purpose::Work {
            return Err(Refusal::because(format!(
                "{lease} is a work lease; a worker runs only a review lease for now. Next: start the agent with its lease."
            )));
        }
        let deadline = unused.deadline_from(at);
        let time = deadline.time().ok_or_else(|| {
            Refusal::because(format!(
                "{lease} has no deadline, and a worker must have one. Next: start the reviewer with its lease."
            ))
        })?;
        let running = self.running_workers().into_iter().map(|worker| worker.lease).collect();
        Ok(Starting { lease, reissue: reissue.deadline(deadline), deadline: time, running })
    }

    /// The workers whose end is not recorded, whatever became of their tasks.
    fn running_workers(&self) -> Vec<Worker> {
        self.leases.iter().filter_map(|(id, slot)| Self::running_worker(*id, slot)).collect()
    }

    fn running_worker(lease: LeaseId, slot: &LeaseSlot) -> Option<Worker> {
        let run = slot.running_worker()?.clone();
        let agent = slot.agent()?.clone();
        Some(Worker { lease, agent, run, deadline: slot.deadline() })
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
    use super::Workers;

    const REPO: &str = include_str!("../../../self/workers.toml");

    #[test]
    fn the_repo_gives_reviewers_four_tools_denies_spawning_and_runs_at_most_4_workers() {
        let workers = Workers::parse(REPO).unwrap();
        assert_eq!(workers.concurrency.running, 4);
        assert_eq!(super::Tools::rules(&workers.review.allow), "Read,Grep,Glob,Bash");
        assert_eq!(super::Tools::rules(&workers.review.deny), "Agent,Task,Bash(claude *),mcp__*");
    }

    #[test]
    fn a_tool_or_limit_with_no_source_does_not_parse() {
        let limit = "[concurrency]\nrunning = 4\n[review]\nallow = []\ndeny = []\n";
        let tool = "[concurrency]\nrunning = 4\nsource = \"s\"\n[[review.allow]]\ntool = \"Read\"\n[[review.deny]]\ntool = \"Agent\"\nsource = \"s\"\n";
        let empty = "[concurrency]\nrunning = 4\nsource = \"s\"\n[[review.allow]]\ntool = \"Read\"\nsource = \" \"\n[review]\ndeny = []\n";
        assert!([limit, tool, empty].iter().all(|text| Workers::parse(text).is_err()));
    }
}
