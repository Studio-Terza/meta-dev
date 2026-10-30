use meta_core::{
    Accepted, AgentName, Clearance, Corrupt, Escape, Event, Hook, Ledger, Malformed, PrNumber, Reason, Record, Refusal,
    Rules, TaskId, Timestamp, Title, Unreadable,
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
            ["open", title @ ..] => ledger(title.join(" ").parse().map(LedgerCall::Open)),
            ["claim", task, "--agent", agent] => {
                ledger(task.parse().and_then(|task| agent.parse().map(|agent| LedgerCall::Claim { task, agent })))
            }
            ["link", task, pr] => {
                ledger(task.parse().and_then(|task| pr.parse().map(|pr| LedgerCall::Link { task, pr })))
            }
            ["merge", pr] => ledger(pr.parse().map(LedgerCall::Merge)),
            ["status"] => ledger(Ok(LedgerCall::Status)),
            ["escape", reason @ ..] => Some(reason.join(" ").parse().map(Self::Escape)),
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
            let outcome = log.transact(|records| {
                let decided =
                    Ledger::fold(&records).map_err(Failure::from).and_then(|ledger| decide(call, ledger, git, github));
                match decided {
                    Ok(accepted) => {
                        let (events, text) = accepted.into_parts();
                        (stamped(events, at), Ok(text))
                    }
                    Err(failure) => (Vec::new(), Err(failure)),
                }
            });
            match outcome.map_err(Failure::from).and_then(|decided| decided) {
                Ok(text) => Answer::done(text),
                Err(failure) => Answer::failed(failure),
            }
        }
    }
}

fn hook<L: Log>(hook: Hook, log: &mut L, at: Timestamp) -> Answer {
    match hook {
        Hook::Settled(accepted) => {
            let (events, text) = accepted.into_parts();
            match log.append(&stamped(events, at)) {
                Ok(()) => Answer::done(text),
                Err(error) => Answer::broken_hook(error),
            }
        }
        Hook::Agent(start) => {
            let outcome = log.transact(|records| match Ledger::fold(&records) {
                Ok(ledger) => {
                    let (events, text) = ledger.admit(start).into_parts();
                    (stamped(events, at), Ok(text))
                }
                Err(corrupt) => (Vec::new(), Err(corrupt.to_string())),
            });
            match outcome.map_err(|error| error.to_string()).and_then(|admitted| admitted) {
                Ok(text) => Answer::done(text),
                Err(why) => Answer::broken_hook(why),
            }
        }
    }
}

fn decide<G: Git, H: GitHub>(call: LedgerCall, ledger: Ledger, git: &G, github: &mut H) -> Result<Accepted, Failure> {
    Ok(match call {
        LedgerCall::Open(title) => ledger.open(title),
        LedgerCall::Claim { task, agent } => {
            let claim = ledger.claim(task, agent)?;
            let rules = Rules::parse(&git.rules()?)?;
            claim.brief(&git.origin()?, &rules)
        }
        LedgerCall::Link { task, pr } => {
            let linking = ledger.link(task, pr)?;
            linking.pin(&github.pull(pr)?)?
        }
        LedgerCall::Merge(pr) => match ledger.merge(pr)?.clear(&github.pull(pr)?)? {
            Clearance::Unmerged(cleared) => {
                github.squash_merge(&cleared)?;
                cleared.land().record()
            }
            Clearance::AlreadyMerged(landed) => landed.record(),
        },
        LedgerCall::Status => ledger.status(),
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

impl From<Corrupt> for Failure {
    fn from(corrupt: Corrupt) -> Self {
        Self::Broken(corrupt.to_string())
    }
}

impl From<Unreadable> for Failure {
    fn from(unreadable: Unreadable) -> Self {
        Self::Broken(unreadable.to_string())
    }
}
