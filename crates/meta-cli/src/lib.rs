use meta_core::{
    Accepted, AgentName, Corrupt, Ledger, Malformed, PrNumber, Reason, Record, Refusal, Rules, TaskId, Title,
    Unreadable,
};
use meta_ports::{Clock, Git, GitHub, Log, PortError, Ports};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Call {
    Open(Title),
    Claim { task: TaskId, agent: AgentName },
    Link { task: TaskId, pr: PrNumber },
    Merge(PrNumber),
    Escape(Reason),
    Status,
    PreToolUse(String),
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

impl Call {
    const USAGE: &str = "usage:\n  meta open <title>\n  meta claim <task> --agent <name>\n  meta link <task> <pr>\n  \
                         meta merge <pr>\n  meta escape <reason>\n  meta status\n  meta hook pre-tool-use < input\n";

    pub fn parse(args: &[String], stdin: impl FnOnce() -> String) -> Result<Self, Answer> {
        let words: Vec<&str> = args.iter().map(String::as_str).collect();
        let call: Option<Result<Self, Malformed>> = match words.as_slice() {
            ["open", title @ ..] => Some(title.join(" ").parse().map(Self::Open)),
            ["claim", task, "--agent", agent] => {
                Some(task.parse().and_then(|task| agent.parse().map(|agent| Self::Claim { task, agent })))
            }
            ["link", task, pr] => Some(task.parse().and_then(|task| pr.parse().map(|pr| Self::Link { task, pr }))),
            ["merge", pr] => Some(pr.parse().map(Self::Merge)),
            ["escape", reason @ ..] => Some(reason.join(" ").parse().map(Self::Escape)),
            ["status"] => Some(Ok(Self::Status)),
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
    let hook = matches!(call, Call::PreToolUse(_));
    match decide(call, ports).and_then(|accepted| record(accepted, ports)) {
        Ok(text) => Answer { text, code: Code::Done },
        Err(Failure::Broken(why)) if hook => Answer {
            text: format!("{}\n", Ledger::denial(&format!("meta is broken ({why}); denied, fail closed."))),
            code: Code::Done,
        },
        Err(Failure::Refused(refusal)) => Answer { text: format!("{refusal}\n"), code: Code::Refused },
        Err(Failure::Broken(why)) => Answer { text: format!("error: {why}\n"), code: Code::Failed },
    }
}

fn decide<L: Log, G: Git, H: GitHub, C: Clock>(call: Call, ports: &mut Ports<L, G, H, C>) -> Result<Accepted, Failure> {
    let ledger = Ledger::fold(&ports.log.read()?)?;
    Ok(match call {
        Call::Open(title) => ledger.open(title),
        Call::Claim { task, agent } => {
            let claim = ledger.claim(task, agent)?;
            let rules = Rules::parse(&ports.git.rules()?)?;
            claim.brief(&ports.git.origin()?, &rules)
        }
        Call::Link { task, pr } => {
            let linking = ledger.link(task, pr)?;
            let head = ports.github.pull(linking.pr())?.head;
            linking.pin(head)
        }
        Call::Merge(pr) => {
            let merging = ledger.merge(pr)?;
            let cleared = merging.clear(&ports.github.pull(pr)?)?;
            ports.github.squash_merge(&cleared)?;
            cleared.merged()
        }
        Call::Escape(reason) => ledger.escape(reason),
        Call::Status => Accepted { events: Vec::new(), text: ledger.status() },
        Call::PreToolUse(input) => ledger.pre_tool_use(&input),
    })
}

fn record<L: Log, G: Git, H: GitHub, C: Clock>(
    accepted: Accepted,
    ports: &mut Ports<L, G, H, C>,
) -> Result<String, Failure> {
    let at = ports.clock.now();
    let records: Vec<Record> = accepted.events.into_iter().map(|event| Record { at, event }).collect();
    ports.log.append(&records)?;
    Ok(accepted.text)
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
