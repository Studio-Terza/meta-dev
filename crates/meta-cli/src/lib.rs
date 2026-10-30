use meta_core::{
    Accepted, AgentName, Cause, Claimant, Clearance, Escape, EscapeId, EscapeKind, EscapeKinds, Event, Hook, Ledger,
    Malformed, Merges, Origin, PrNumber, Predictions, Pull, Reason, Reconciliation, Record, Refusal, Rules, SelfFile,
    Sources, TaskId, Timestamp, Title, Unreached, Unreadable, Verdict,
};
use meta_ports::{Clock, Git, GitHub, Log, PortError, Ports};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Call {
    Ledger(LedgerCall),
    Escape { kind: EscapeKind, reason: Reason },
    PreToolUse(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LedgerCall {
    Local(Local),
    Claim { task: TaskId, claimant: Claimant },
    Link { task: TaskId, pr: PrNumber },
    Merge(PrNumber),
    Retire { task: TaskId, reason: Reason },
    Review { task: TaskId, agent: AgentName },
    Status,
}

/// A ledger command decided from the log alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Local {
    Open(Title),
    Resolve { escape: EscapeId, task: TaskId },
    Defer { task: TaskId, reason: Reason },
    Undefer(TaskId),
    Verdict { task: TaskId, verdict: Verdict },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Code {
    Done,
    Refused,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    pub text: String,
    pub code: Code,
}

enum Failure {
    Refused(Refusal),
    Broken(String),
}

impl Code {
    pub fn byte(self) -> u8 {
        match self {
            Self::Done => 0,
            Self::Refused => 1,
            Self::Failed => 2,
        }
    }
}

impl Answer {
    fn done(text: String) -> Self {
        Self { text, code: Code::Done }
    }

    fn failed(failure: Failure) -> Self {
        match failure {
            Failure::Refused(refusal) => Self { text: format!("{refusal}\n"), code: Code::Refused },
            Failure::Broken(why) => Self { text: format!("error: {why}\n"), code: Code::Failed },
        }
    }

    fn broken_hook(why: impl ToString) -> Self {
        Self::done(Hook::broken(&format!("meta is broken ({})", why.to_string())).into_parts().1)
    }
}

impl Call {
    const USAGE: &str = "usage:\n  meta open <title>\n  meta claim <task> --agent <name>\n  meta claim <task> --self\n  \
                         meta link <task> <pr>\n  meta merge <pr>\n  meta escape <kind> <reason>\n  \
                         meta resolve <escape> <task>\n  meta retire <task> <reason>\n  meta defer <task> <reason>\n  \
                         meta undefer <task>\n  meta review <task> --agent <name>\n  meta verdict <task> pass|fail\n  \
                         meta status\n  meta hook pre-tool-use < input\n";

    pub fn parse(args: &[String], stdin: impl FnOnce() -> String) -> Result<Self, Answer> {
        let words: Vec<&str> = args.iter().map(String::as_str).collect();
        let ledger = |call: Result<LedgerCall, Malformed>| Some(call.map(Self::Ledger));
        let local = |call: Result<Local, Malformed>| ledger(call.map(LedgerCall::Local));
        let call: Option<Result<Self, Malformed>> = match words.as_slice() {
            ["open", title @ ..] => local(title.join(" ").parse().map(Local::Open)),
            ["claim", task, "--self"] => {
                ledger(task.parse().map(|task| LedgerCall::Claim { task, claimant: Claimant::Lead }))
            }
            ["claim", task, "--agent", agent] => ledger(task.parse().and_then(|task| {
                agent.parse().map(|agent| LedgerCall::Claim { task, claimant: Claimant::Agent(agent) })
            })),
            ["link", task, pr] => {
                ledger(task.parse().and_then(|task| pr.parse().map(|pr| LedgerCall::Link { task, pr })))
            }
            ["merge", pr] => ledger(pr.parse().map(LedgerCall::Merge)),
            ["status"] => ledger(Ok(LedgerCall::Status)),
            ["escape", kind, reason @ ..] => {
                Some(kind.parse().and_then(|kind| reason.join(" ").parse().map(|reason| Self::Escape { kind, reason })))
            }
            ["resolve", escape, task] => {
                local(escape.parse().and_then(|escape| task.parse().map(|task| Local::Resolve { escape, task })))
            }
            ["retire", task, reason @ ..] => ledger(
                task.parse()
                    .and_then(|task| reason.join(" ").parse().map(|reason| LedgerCall::Retire { task, reason })),
            ),
            ["defer", task, reason @ ..] => local(
                task.parse().and_then(|task| reason.join(" ").parse().map(|reason| Local::Defer { task, reason })),
            ),
            ["undefer", task] => local(task.parse().map(Local::Undefer)),
            ["review", task, "--agent", agent] => {
                ledger(task.parse().and_then(|task| agent.parse().map(|agent| LedgerCall::Review { task, agent })))
            }
            ["verdict", task, verdict] => {
                local(task.parse().and_then(|task| verdict.parse().map(|verdict| Local::Verdict { task, verdict })))
            }
            ["hook", "pre-tool-use"] => Some(Ok(Self::PreToolUse(stdin()))),
            _ => None,
        };
        let why = match call {
            Some(Ok(call)) => return Ok(call),
            Some(Err(malformed)) => malformed.to_string(),
            None => "unknown command".to_owned(),
        };
        Err(Answer { text: format!("{why}\n{}", Self::USAGE), code: Code::Failed })
    }
}

pub fn answer<L: Log, G: Git, H: GitHub, C: Clock>(call: Call, ports: &mut Ports<L, G, H, C>) -> Answer {
    let Ports { log, git, github, clock } = ports;
    let at = clock.now();
    match call {
        Call::PreToolUse(input) => hook(Hook::read(&input), log, at),
        Call::Escape { kind, reason } => {
            let kinds = match spec(git, SelfFile::EscapeKinds, EscapeKinds::parse) {
                Ok(kinds) => kinds,
                Err(failure) => return Answer::failed(failure),
            };
            let earlier = log.read().map_err(|error| error.to_string());
            let tallied = Escape::new(kind, reason).tally(&kinds, earlier.as_deref().map_err(String::as_str));
            let (events, text) = match tallied {
                Ok(accepted) => accepted.into_parts(),
                Err(refusal) => return Answer::failed(refusal.into()),
            };
            match log.append(&stamped(events, at)) {
                Ok(()) => Answer::done(text),
                Err(error) => Answer::failed(error.into()),
            }
        }
        Call::Ledger(call) => {
            let place = log.place();
            let fold =
                |records: &[Record]| Ledger::fold(records).map_err(|corrupt| Failure::Broken(corrupt.in_log(&place)));
            let prepared = log
                .read()
                .map_err(Failure::from)
                .and_then(|records| fold(&records))
                .and_then(|ledger| prepare(call, &ledger, git, github));
            let prepared = match prepared {
                Ok(prepared) => prepared,
                Err(failure) => return Answer::failed(failure),
            };
            let outcome = log.transact(|records| match fold(&records).and_then(|ledger| finish(prepared, ledger)) {
                Ok(accepted) => {
                    let (events, text) = accepted.into_parts();
                    (stamped(events, at), Ok(text))
                }
                Err(failure) => (Vec::new(), Err(failure)),
            });
            match outcome.map_err(Failure::from).and_then(|decided| decided) {
                Ok(text) => Answer::done(text),
                Err(failure) => Answer::failed(failure),
            }
        }
    }
}

fn hook<L: Log>(hook: Hook, log: &mut L, at: Timestamp) -> Answer {
    let decide: Box<dyn FnOnce(Ledger) -> Accepted> = match hook {
        Hook::Settled(accepted) => return Answer::done(accepted.into_parts().1),
        Hook::Agent(start) => Box::new(|ledger: Ledger| ledger.admit(start)),
        Hook::Resume(resume) => Box::new(|ledger: Ledger| ledger.resume(resume)),
    };
    let place = log.place();
    let outcome = log.transact(|records| match Ledger::fold(&records) {
        Ok(ledger) => {
            let (events, text) = decide(ledger).into_parts();
            (stamped(events, at), Ok(text))
        }
        Err(corrupt) => (Vec::new(), Err(corrupt.in_log(&place))),
    });
    match outcome.map_err(|error| error.to_string()).and_then(|decided| decided) {
        Ok(text) => Answer::done(text),
        Err(why) => Answer::broken_hook(why),
    }
}

/// A `self/` file read from main and parsed; a failure is an error, not a refusal.
fn spec<G: Git, T>(git: &G, file: SelfFile, parse: fn(&str) -> Result<T, Unreadable>) -> Result<T, Failure> {
    Ok(parse(&git.read(file)?)?)
}

enum Prepared {
    Local(Local),
    Status(Sources),
    Claim { task: TaskId, claimant: Claimant, origin: Origin, rules: Rules },
    Link { task: TaskId, pr: PrNumber, pull: Pull },
    Merge { pr: PrNumber, pull: Pull },
    Retire { task: TaskId, reason: Reason, pull: Option<Pull> },
    Review { task: TaskId, agent: AgentName, pull: Pull, rules: Rules },
}

fn prepare<G: Git, H: GitHub>(call: LedgerCall, ledger: &Ledger, git: &G, github: &mut H) -> Result<Prepared, Failure> {
    Ok(match call {
        LedgerCall::Local(local) => Prepared::Local(local),
        LedgerCall::Status => {
            let unreached = |failure: Failure| Unreached::by(&failure);
            Prepared::Status(Sources {
                github: match github.merged_list() {
                    Ok(listed) => Merges::from_gh(&listed),
                    Err(error) => Reconciliation::Unavailable(Cause::Port(Unreached::by(&error))),
                },
                kinds: spec(git, SelfFile::EscapeKinds, EscapeKinds::parse).map_err(unreached),
                predictions: spec(git, SelfFile::Predictions, Predictions::parse).map_err(unreached),
            })
        }
        LedgerCall::Claim { task, claimant } => {
            ledger.claim(task, claimant.clone())?;
            let rules = spec(git, SelfFile::Rules, Rules::parse)?;
            Prepared::Claim { task, claimant, origin: git.origin()?, rules }
        }
        LedgerCall::Link { task, pr } => {
            ledger.link(task, pr)?;
            Prepared::Link { task, pr, pull: github.pull(pr)? }
        }
        LedgerCall::Merge(pr) => {
            let merging = ledger.merge(pr)?;
            let pull = github.pull(pr)?;
            if let Clearance::Unmerged(cleared) = merging.clear(&pull)? {
                github.squash_merge(&cleared)?;
            }
            Prepared::Merge { pr, pull }
        }
        LedgerCall::Retire { task, reason } => {
            let pull = ledger.retire(task, reason.clone())?.pr().map(|pr| github.pull(pr)).transpose()?;
            Prepared::Retire { task, reason, pull }
        }
        LedgerCall::Review { task, agent } => {
            let pull = github.pull(ledger.review(task, agent.clone())?.pr())?;
            Prepared::Review { task, agent, pull, rules: spec(git, SelfFile::Rules, Rules::parse)? }
        }
    })
}

fn finish(prepared: Prepared, ledger: Ledger) -> Result<Accepted, Failure> {
    Ok(match prepared {
        Prepared::Local(Local::Open(title)) => ledger.open(title),
        Prepared::Local(Local::Resolve { escape, task }) => ledger.resolve(escape, task)?,
        Prepared::Local(Local::Defer { task, reason }) => ledger.defer(task, reason)?,
        Prepared::Local(Local::Undefer(task)) => ledger.undefer(task)?,
        Prepared::Local(Local::Verdict { task, verdict }) => ledger.verdict(task, verdict)?,
        Prepared::Status(sources) => ledger.status(sources),
        Prepared::Claim { task, claimant, origin, rules } => ledger.claim(task, claimant)?.brief(&origin, &rules),
        Prepared::Link { task, pr, pull } => ledger.link(task, pr)?.pin(&pull)?,
        Prepared::Merge { pr, pull } => match ledger.merge(pr)?.clear(&pull)? {
            Clearance::Unmerged(cleared) => cleared.land().record(),
            Clearance::AlreadyMerged(landed) => landed.record(),
        },
        Prepared::Retire { task, reason, pull } => ledger.retire(task, reason)?.settle(pull.as_ref())?,
        Prepared::Review { task, agent, pull, rules } => ledger.review(task, agent)?.brief(&pull, &rules)?,
    })
}

fn stamped(events: Vec<Event>, at: Timestamp) -> Vec<Record> {
    events.into_iter().map(|event| Record { at, event }).collect()
}

impl From<Refusal> for Failure {
    fn from(refusal: Refusal) -> Self {
        Self::Refused(refusal)
    }
}

impl From<PortError> for Failure {
    fn from(error: PortError) -> Self {
        Self::Broken(error.to_string())
    }
}

impl std::fmt::Display for Failure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Refused(refusal) => write!(formatter, "{refusal}"),
            Self::Broken(why) => formatter.write_str(why),
        }
    }
}

impl From<Unreadable> for Failure {
    fn from(unreadable: Unreadable) -> Self {
        Self::Broken(unreadable.to_string())
    }
}
