use meta_core::{
    Accepted, AgentName, Cause, Clearance, Escape, Event, Hook, Ledger, Malformed, Merges, Origin, PrNumber, Pull,
    Reason, Reconciliation, Record, Refusal, Rules, TaskId, Timestamp, Title, Unreached, Unreadable,
};
use meta_ports::{Clock, Git, GitHub, Log, PortError, Ports};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Call {
    Ledger(LedgerCall),
    Escape(Reason),
    PreToolUse(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LedgerCall {
    Open(Title),
    Claim { task: TaskId, agent: AgentName },
    Link { task: TaskId, pr: PrNumber },
    Merge(PrNumber),
    Status,
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
    const USAGE: &str = "usage:\n  meta open <title>\n  meta claim <task> --agent <name>\n  meta link <task> <pr>\n  \
                         meta merge <pr>\n  meta escape <reason>\n  meta status\n  meta hook pre-tool-use < input\n";

    pub fn parse(args: &[String], stdin: impl FnOnce() -> String) -> Result<Self, Answer> {
        let words: Vec<&str> = args.iter().map(String::as_str).collect();
        let ledger = |call: Result<LedgerCall, Malformed>| Some(call.map(Self::Ledger));
        let call: Option<Result<Self, Malformed>> = match words.as_slice() {
            ["open", title @ ..] => ledger(spaced(title).parse().map(LedgerCall::Open)),
            ["claim", task, "--agent", agent] => {
                ledger(task.parse().and_then(|task| agent.parse().map(|agent| LedgerCall::Claim { task, agent })))
            }
            ["link", task, pr] => {
                ledger(task.parse().and_then(|task| pr.parse().map(|pr| LedgerCall::Link { task, pr })))
            }
            ["merge", pr] => ledger(pr.parse().map(LedgerCall::Merge)),
            ["status"] => ledger(Ok(LedgerCall::Status)),
            ["escape", reason @ ..] => Some(spaced(reason).parse().map(Self::Escape)),
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
        Call::Escape(reason) => {
            let earlier = log.read().map_err(|error| error.to_string());
            let (events, text) = Escape::new(reason).tally(earlier.as_deref().map_err(String::as_str)).into_parts();
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

fn spaced(words: &[&str]) -> String {
    words.iter().flat_map(|word| [" ", word]).skip(1).collect()
}

fn hook<L: Log>(hook: Hook, log: &mut L, at: Timestamp) -> Answer {
    match hook {
        Hook::Settled(accepted) => Answer::done(accepted.into_parts().1),
        Hook::Agent(start) => {
            let place = log.place();
            let outcome = log.transact(|records| match Ledger::fold(&records) {
                Ok(ledger) => {
                    let (events, text) = ledger.admit(start).into_parts();
                    (stamped(events, at), Ok(text))
                }
                Err(corrupt) => (Vec::new(), Err(corrupt.in_log(&place))),
            });
            match outcome.map_err(|error| error.to_string()).and_then(|admitted| admitted) {
                Ok(text) => Answer::done(text),
                Err(why) => Answer::broken_hook(why),
            }
        }
    }
}

enum Prepared {
    Open(Title),
    Status(Reconciliation),
    Claim { task: TaskId, agent: AgentName, origin: Origin, rules: Rules },
    Link { task: TaskId, pr: PrNumber, pull: Pull },
    Merge { pr: PrNumber, pull: Pull },
}

fn prepare<G: Git, H: GitHub>(call: LedgerCall, ledger: &Ledger, git: &G, github: &mut H) -> Result<Prepared, Failure> {
    Ok(match call {
        LedgerCall::Open(title) => Prepared::Open(title),
        LedgerCall::Status => Prepared::Status(match github.merged_list() {
            Ok(listed) => Merges::from_gh(&listed),
            Err(error) => Reconciliation::Unavailable(Cause::Port(Unreached::by(&error))),
        }),
        LedgerCall::Claim { task, agent } => {
            ledger.claim(task, agent.clone())?;
            let rules = Rules::parse(&git.rules()?)?;
            Prepared::Claim { task, agent, origin: git.origin()?, rules }
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
    })
}

fn finish(prepared: Prepared, ledger: Ledger) -> Result<Accepted, Failure> {
    Ok(match prepared {
        Prepared::Open(title) => ledger.open(title),
        Prepared::Status(github) => ledger.status(github),
        Prepared::Claim { task, agent, origin, rules } => ledger.claim(task, agent)?.brief(&origin, &rules),
        Prepared::Link { task, pr, pull } => ledger.link(task, pr)?.pin(&pull)?,
        Prepared::Merge { pr, pull } => match ledger.merge(pr)?.clear(&pull)? {
            Clearance::Unmerged(cleared) => cleared.land().record(),
            Clearance::AlreadyMerged(landed) => landed.record(),
        },
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

impl From<Unreadable> for Failure {
    fn from(unreadable: Unreadable) -> Self {
        Self::Broken(unreadable.to_string())
    }
}
