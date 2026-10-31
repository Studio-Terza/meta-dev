use meta_core::{
    Accepted, Admitted, AgentName, Build, Cause, Claimant, Clearance, Escape, EscapeId, EscapeKind, EscapeKinds, Event,
    Hook, Importing, LeaseId, Ledger, Malformed, Marking, Merges, Minutes, Origin, OwnerAnswer, OwnerQuestion, Phase,
    PrNumber, Predictions, Pull, Reason, Reconciliation, Record, Refusal, Role, Roles, Rules, RunCount, SelfFile, Sha,
    Sources, Span, Standing, TaskId, Timestamp, Title, Unreached, Unreadable, Unverified, Updating, Verdict,
};
use meta_ports::{Clock, Git, GitHub, Log, PortError, Ports};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Call {
    Ledger(LedgerCall),
    Escape {
        kind: EscapeKind,
        reason: Reason,
        pr: Option<PrNumber>,
    },
    /// A Claude Code hook, before or after a tool runs, with its input.
    Hook(Phase, String),
    /// Moves the side file's escapes into the log with ids; only a main build may.
    ImportEscapes,
    /// Puts the `RealLog` marker first in the log, once; only a main build may.
    MarkRealLog,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LedgerCall {
    Local(Local),
    /// `deadline` overrides the work default; the parse gives one only to an agent's claim.
    Claim {
        task: TaskId,
        claimant: Claimant,
        deadline: Option<Minutes>,
    },
    Link {
        task: TaskId,
        pr: PrNumber,
    },
    Merge(PrNumber),
    /// Merges main into a linked PR's branch when it is behind; never merges the PR.
    Update(PrNumber),
    Retire {
        task: TaskId,
        reason: Reason,
    },
    /// `deadline` overrides the review default.
    Review {
        task: TaskId,
        agent: AgentName,
        deadline: Option<Minutes>,
    },
    /// Prints again the brief of an unused, live lease; records nothing.
    Brief(LeaseId),
    Status,
}

/// A ledger command decided from the log alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Local {
    Open(Title),
    Resolve { escape: EscapeId, task: TaskId },
    Historical { escape: EscapeId, reason: Reason, correcting: bool },
    Defer { task: TaskId, reason: Reason },
    Undefer(TaskId),
    Role { task: TaskId, role: Role },
    Verdict { task: TaskId, verdict: Verdict },
    Wait { task: TaskId, question: OwnerQuestion },
    Unwait { task: TaskId, answer: OwnerAnswer },
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

    fn broken_hook(phase: Phase, why: impl ToString) -> Self {
        Self::done(phase.broken(&format!("meta is broken ({})", why.to_string())).into_parts().1)
    }
}

impl Call {
    const USAGE: &str = "usage:\n  meta open <title>\n  meta claim <task> --agent <name> [--deadline <minutes>]\n  \
                         meta claim <task> --self\n  \
                         meta link <task> <pr>\n  meta merge <pr> [--update]\n  meta escape <kind> [--pr <pr>] <reason>\n  meta escape --import\n  \
                         meta resolve <escape> <task>\n  meta resolve <escape> --historical <reason> [--correcting]\n  meta retire <task> <reason>\n  meta defer <task> <reason>\n  \
                         meta undefer <task>\n  meta role <task> critic\n  meta review <task> --agent <name> [--deadline <minutes>]\n  \
                         meta verdict <task> pass|fail\n  meta brief <lease>\n  \
                         meta wait <task> --owner <question>\n  meta unwait <task> <answer>\n  \
                         meta status\n  meta mark-real-log\n  meta hook pre-tool-use < input\n  meta hook post-tool-use < input\n";

    pub fn parse(args: &[String], stdin: impl FnOnce() -> String) -> Result<Self, Answer> {
        let words: Vec<&str> = args.iter().map(String::as_str).collect();
        let ledger = |call: Result<LedgerCall, Malformed>| Some(call.map(Self::Ledger));
        let local = |call: Result<Local, Malformed>| ledger(call.map(LedgerCall::Local));
        let call: Option<Result<Self, Malformed>> = match words.as_slice() {
            ["open", title @ ..] => local(title.join(" ").parse().map(Local::Open)),
            ["claim", task, "--self"] => {
                ledger(task.parse().map(|task| LedgerCall::Claim { task, claimant: Claimant::Lead, deadline: None }))
            }
            ["claim", task, "--agent", agent, deadline @ ..] => ledger(task.parse().and_then(|task| {
                let deadline = Self::deadline(deadline)?;
                agent.parse().map(|agent| LedgerCall::Claim { task, claimant: Claimant::Agent(agent), deadline })
            })),
            ["link", task, pr] => {
                ledger(task.parse().and_then(|task| pr.parse().map(|pr| LedgerCall::Link { task, pr })))
            }
            ["merge", pr] => ledger(pr.parse().map(LedgerCall::Merge)),
            ["merge", pr, "--update"] => ledger(pr.parse().map(LedgerCall::Update)),
            ["brief", lease] => ledger(lease.parse().map(LedgerCall::Brief)),
            ["status"] => ledger(Ok(LedgerCall::Status)),
            ["mark-real-log"] => Some(Ok(Self::MarkRealLog)),
            ["escape", "--import"] => Some(Ok(Self::ImportEscapes)),
            ["escape", kind, "--pr", pr, reason @ ..] => Some(kind.parse().and_then(|kind| {
                pr.parse()
                    .and_then(|pr| reason.join(" ").parse().map(|reason| Self::Escape { kind, reason, pr: Some(pr) }))
            })),
            ["escape", kind, reason @ ..] => Some(
                kind.parse()
                    .and_then(|kind| reason.join(" ").parse().map(|reason| Self::Escape { kind, reason, pr: None })),
            ),
            ["resolve", escape, "--historical", reason @ .., "--correcting"] => {
                local(escape.parse().and_then(|escape| {
                    reason.join(" ").parse().map(|reason| Local::Historical { escape, reason, correcting: true })
                }))
            }
            ["resolve", escape, "--historical", reason @ ..] => local(escape.parse().and_then(|escape| {
                reason.join(" ").parse().map(|reason| Local::Historical { escape, reason, correcting: false })
            })),
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
            ["role", task, role] => {
                local(task.parse().and_then(|task| role.parse().map(|role| Local::Role { task, role })))
            }
            ["review", task, "--agent", agent, deadline @ ..] => ledger(task.parse().and_then(|task| {
                let deadline = Self::deadline(deadline)?;
                agent.parse().map(|agent| LedgerCall::Review { task, agent, deadline })
            })),
            ["verdict", task, verdict] => {
                local(task.parse().and_then(|task| verdict.parse().map(|verdict| Local::Verdict { task, verdict })))
            }
            ["wait", task, "--owner", question @ ..] => local(
                task.parse().and_then(|task| question.join(" ").parse().map(|question| Local::Wait { task, question })),
            ),
            ["unwait", task, answer @ ..] => local(
                task.parse().and_then(|task| answer.join(" ").parse().map(|answer| Local::Unwait { task, answer })),
            ),
            ["hook", "pre-tool-use"] => Some(Ok(Self::Hook(Phase::Before, stdin()))),
            ["hook", "post-tool-use"] => Some(Ok(Self::Hook(Phase::After, stdin()))),
            _ => None,
        };
        let why = match call {
            Some(Ok(call)) => return Ok(call),
            Some(Err(malformed)) => malformed.to_string(),
            None => "unknown command".to_owned(),
        };
        Err(Answer { text: format!("{why}\n{}", Self::USAGE), code: Code::Failed })
    }

    /// The optional `--deadline <minutes>` that ends a claim or review by an agent.
    fn deadline(words: &[&str]) -> Result<Option<Minutes>, Malformed> {
        match words {
            [] => Ok(None),
            ["--deadline", minutes] => minutes.parse().map(Some),
            _ => Err(Malformed::because(format!("{words:?} is not --deadline <minutes>"))),
        }
    }
}

pub fn answer<L: Log, G: Git, H: GitHub, C: Clock>(call: Call, ports: &mut Ports<L, G, H, C>) -> Answer {
    let Ports { log, git, github, clock, build } = ports;
    let at = clock.now();
    let standing = log.standing();
    let mut guard = Guard::default();
    match call {
        Call::Hook(phase, input) => {
            hook(phase, Hook::read(phase, &input), log, at, standing, |standing| guard.at(standing, git, build))
        }
        Call::MarkRealLog => {
            if let Err(unverified) = build.verify_marking(|commit| git.on_main(commit)) {
                return Answer::failed(Failure::Refused(unverified.into()));
            }
            match log.mark(Record { at, event: Event::RealLog }) {
                Ok(Marking::Added) => Answer::done(format!(
                    "marked {} as the real log: only a main build writes it now, by any path\n",
                    log.place()
                )),
                Ok(Marking::AlreadyMarked) => {
                    Answer::done(format!("{} is already marked as the real log\n", log.place()))
                }
                Err(error) => Answer::failed(error.into()),
            }
        }
        Call::ImportEscapes => {
            if let Err(unverified) = guard.at(standing, git, build) {
                return Answer::failed(Failure::Refused(unverified.into()));
            }
            // Checked again inside the lock, as for an escape. The side file is rewritten only
            // after the appended records are durable.
            let imported = log.import_set_aside(|now, records, lines| {
                if let Err(unverified) = guard.at(now, git, build) {
                    return (Vec::new(), None, Err(Failure::Refused(unverified.into())));
                }
                match records {
                    Ok(records) => {
                        let (events, kept, text) = Importing::of(lines, &records).into_parts();
                        (stamped(events, at), Some(kept), Ok(text))
                    }
                    Err(error) => (Vec::new(), None, Err(error.into())),
                }
            });
            match imported.map_err(Failure::from).and_then(|decided| decided) {
                Ok(text) => Answer::done(text),
                Err(failure) => Answer::failed(failure),
            }
        }
        Call::Escape { kind, reason, pr } => {
            let kinds = git
                .local(SelfFile::EscapeKinds)
                .map_err(Failure::from)
                .and_then(|text| Ok(EscapeKinds::parse(&text)?))
                .map_err(|failure| Unreached::by(&failure));
            let admitted = match Escape::new(kind, reason, pr).admit(kinds) {
                Ok(admitted) => admitted,
                Err(refusal) => return Answer::failed(refusal.into()),
            };
            let set_aside = |log: &mut L, admitted: Admitted, unverified: Unverified| {
                let (escape, text) = admitted.set_aside(at);
                match log.set_aside(&escape) {
                    Ok(side) => Answer::done(format!("{text}kept in {side}, not in the log: {unverified}\n")),
                    Err(error) => Answer::failed(error.into()),
                }
            };
            if let Err(unverified) = guard.at(standing, git, build) {
                return set_aside(log, admitted, unverified);
            }
            // The log's first record is read again inside the lock: a marking that landed since
            // `standing` was read turns this append into a set-aside.
            let recorded = log.transact(|now, records| match guard.at(now, git, build) {
                Ok(()) => {
                    let earlier = records.as_deref().map_err(ToString::to_string);
                    let (events, text) = admitted.tally(earlier).into_parts();
                    (stamped(events, at), Ok(text))
                }
                Err(unverified) => (Vec::new(), Err((admitted, unverified))),
            });
            match recorded {
                Ok(Ok(text)) => Answer::done(text),
                Ok(Err((admitted, unverified))) => set_aside(log, admitted, unverified),
                Err(error) => Answer::failed(error.into()),
            }
        }
        Call::Ledger(call) => {
            let verified = guard.at(standing, git, build);
            let reading = call == LedgerCall::Status;
            if let (false, Err(unverified)) = (reading, &verified) {
                return Answer::failed(Failure::Refused(unverified.clone().into()));
            }
            let place = log.place();
            let fold =
                |records: &[Record]| Ledger::fold(records).map_err(|corrupt| Failure::Broken(corrupt.in_log(&place)));
            let prepared = log
                .read()
                .map_err(Failure::from)
                .and_then(|records| fold(&records))
                .and_then(|ledger| prepare(call, &ledger, log, git, github, clock));
            let prepared = match prepared {
                Ok(prepared) => prepared,
                Err(failure) => return Answer::failed(failure),
            };
            let outcome = log.transact(|now, records| {
                // Rechecked against the first record inside the lock, as for an escape.
                let verified = guard.at(now, git, build);
                if let (false, Err(unverified)) = (reading, &verified) {
                    return (Vec::new(), Err(Failure::Refused(unverified.clone().into())));
                }
                match records
                    .map_err(Failure::from)
                    .and_then(|records| fold(&records))
                    .and_then(|ledger| finish(prepared, ledger, at))
                {
                    Ok(accepted) => match (accepted.into_parts(), &verified) {
                        ((events, text), Ok(())) => (stamped(events, at), Ok(text)),
                        ((events, text), Err(_)) if events.is_empty() => (Vec::new(), Ok(text)),
                        ((_, text), Err(unverified)) => (Vec::new(), Ok(format!("{text}not recorded: {unverified}\n"))),
                    },
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

/// Whether this binary may append to the log: any build may write a scratch log, and only a build
/// from a commit on main may write the real one, which the log's first record names
/// (`Log::standing`). An ancestry that cannot be decided is unverified. The standing is read once
/// before the lock and again inside it, so the ancestry is asked at most once, and not at all for
/// a log that stays scratch.
#[derive(Default)]
struct Guard {
    real: Option<Result<(), Unverified>>,
}

impl Guard {
    fn at<G: Git>(&mut self, standing: Standing, git: &G, build: &Build) -> Result<(), Unverified> {
        match standing {
            Standing::Scratch => Ok(()),
            Standing::AwaitingMark | Standing::Real | Standing::MarkedElsewhere => self
                .real
                .get_or_insert_with(|| build.verify(Standing::Real, |commit| git.on_main(commit)))
                .clone()
                .map_err(|unverified| unverified.at(standing)),
        }
    }
}

fn hook<L: Log>(
    phase: Phase,
    hook: Hook,
    log: &mut L,
    at: Timestamp,
    standing: Standing,
    mut verify: impl FnMut(Standing) -> Result<(), Unverified>,
) -> Answer {
    let decide: Box<dyn FnOnce(Ledger) -> Accepted> = match hook {
        Hook::Settled(accepted) => return Answer::done(accepted.into_parts().1),
        Hook::Agent(start) => Box::new(|ledger: Ledger| ledger.admit(start)),
        Hook::Resume(resume) => Box::new(|ledger: Ledger| ledger.resume(resume)),
        Hook::Bind(binding) => Box::new(move |ledger: Ledger| ledger.bind(binding, at)),
    };
    if let Err(unverified) = verify(standing) {
        return Answer::broken_hook(phase, unverified);
    }
    let place = log.place();
    let outcome = log.transact(|now, records| {
        // The log's first record is read again inside the lock: a marking that landed since
        // `standing` was read refuses this append.
        if let Err(unverified) = verify(now) {
            return (Vec::new(), Err(unverified.to_string()));
        }
        let folded = records
            .map_err(|error| error.to_string())
            .and_then(|records| Ledger::fold(&records).map_err(|corrupt| corrupt.in_log(&place)));
        match folded {
            Ok(ledger) => {
                let (events, text) = decide(ledger).into_parts();
                (stamped(events, at), Ok(text))
            }
            Err(why) => (Vec::new(), Err(why)),
        }
    });
    match outcome.map_err(|error| error.to_string()).and_then(|decided| decided) {
        Ok(text) => Answer::done(text),
        Err(why) => Answer::broken_hook(phase, why),
    }
}

/// A `self/` file read from main and parsed; a failure is an error, not a refusal.
fn spec<G: Git, T>(git: &G, file: SelfFile, parse: fn(&str) -> Result<T, Unreadable>) -> Result<T, Failure> {
    Ok(parse(&git.read(file)?)?)
}

enum Prepared {
    Local(Local),
    Status(Sources),
    /// `allowance` is how long an agent's lease runs; a self-claim has none.
    Claim {
        task: TaskId,
        claimant: Claimant,
        origin: Origin,
        rules: Rules,
        allowance: Option<Minutes>,
    },
    Link {
        task: TaskId,
        pr: PrNumber,
        pushed: Sha,
        pull: Pull,
    },
    Update {
        updating: Updating,
    },
    Merge {
        pr: PrNumber,
        pull: Pull,
    },
    Retire {
        task: TaskId,
        reason: Reason,
        pull: Option<Pull>,
    },
    Brief {
        lease: LeaseId,
        origin: Origin,
        rules: Rules,
    },
    Review {
        task: TaskId,
        agent: AgentName,
        pull: Pull,
        rules: Rules,
        allowance: Minutes,
    },
}

/// How long `meta link` waits for GitHub to report the commit that was pushed, and how often it asks.
const LINK_DEADLINE: Span = Span::of_seconds(60);
const LINK_POLL: Span = Span::of_seconds(5);

/// The PR as GitHub reports it once its head is `pushed`; when the deadline passes first, the last
/// report, so that the pin refuses with both commits named. A GitHub error before the deadline is
/// asked again; one still standing at the deadline is the failure.
fn pull_at<H: GitHub, C: Clock>(github: &H, clock: &C, pr: PrNumber, pushed: &Sha) -> Result<Pull, Failure> {
    let deadline = clock.now().plus(LINK_DEADLINE);
    loop {
        let reported = github.pull(pr);
        let settled = reported.as_ref().is_ok_and(|pull| pull.is_at(pushed));
        if settled || clock.now() >= deadline {
            return Ok(reported?);
        }
        clock.pause(LINK_POLL);
    }
}

fn prepare<L: Log, G: Git, H: GitHub, C: Clock>(
    call: LedgerCall,
    ledger: &Ledger,
    log: &L,
    git: &G,
    github: &mut H,
    clock: &C,
) -> Result<Prepared, Failure> {
    Ok(match call {
        LedgerCall::Local(local) => Prepared::Local(local),
        LedgerCall::Status => {
            let unreached = |failure: Failure| Unreached::by(&failure);
            let runs = ledger
                .rework_prs()
                .into_iter()
                .map(|pr| {
                    let read = github
                        .runs(pr)
                        .map_err(|error| Unreached::by(&error))
                        .and_then(|listed| RunCount::from_gh(&listed).map_err(|error| Unreached::by(&error)));
                    (pr, read)
                })
                .collect();
            Prepared::Status(Sources {
                runs,
                github: match github.merged_list() {
                    Ok(listed) => Merges::from_gh(&listed),
                    Err(error) => Reconciliation::Unavailable(Cause::Port(Unreached::by(&error))),
                },
                kinds: spec(git, SelfFile::EscapeKinds, EscapeKinds::parse).map_err(unreached),
                predictions: spec(git, SelfFile::Predictions, Predictions::parse).map_err(unreached),
                unverified: log.set_aside_escapes().map_err(|error| Unreached::by(&error)),
                standing: log.standing(),
            })
        }
        LedgerCall::Claim { task, claimant, deadline } => {
            ledger.claim(task, claimant.clone())?;
            let rules = spec(git, SelfFile::Rules, Rules::parse)?;
            let allowance = match claimant {
                Claimant::Lead => None,
                Claimant::Agent(_) => Some(deadline.map_or_else(|| roles(git).map(|roles| roles.work()), Ok)?),
            };
            Prepared::Claim { task, claimant, origin: git.origin()?, rules, allowance }
        }
        LedgerCall::Link { task, pr } => {
            let pushed = ledger.link(task, pr)?.pushed(git.local_head(task)?)?;
            Prepared::Link { task, pr, pull: pull_at(github, clock, pr, &pushed)?, pushed }
        }
        LedgerCall::Merge(pr) => {
            let merging = ledger.merge(pr)?;
            let pull = github.pull(pr)?;
            if let Clearance::Unmerged(cleared) = merging.clear(&pull)? {
                github.squash_merge(&cleared)?;
            }
            Prepared::Merge { pr, pull }
        }
        LedgerCall::Update(pr) => {
            let updating = ledger.merge(pr)?.update(&github.pull(pr)?)?;
            github.update_branch(&updating)?;
            Prepared::Update { updating }
        }
        LedgerCall::Retire { task, reason } => {
            let pull = ledger.retire(task, reason.clone())?.pr().map(|pr| github.pull(pr)).transpose()?;
            Prepared::Retire { task, reason, pull }
        }
        LedgerCall::Brief(lease) => {
            ledger.reissue(lease)?;
            Prepared::Brief { lease, origin: git.origin()?, rules: spec(git, SelfFile::Rules, Rules::parse)? }
        }
        LedgerCall::Review { task, agent, deadline } => {
            let pull = github.pull(ledger.review(task, agent.clone())?.pr())?;
            let allowance = deadline.map_or_else(|| roles(git).map(|roles| roles.review()), Ok)?;
            Prepared::Review { task, agent, pull, rules: spec(git, SelfFile::Rules, Rules::parse)?, allowance }
        }
    })
}

fn finish(prepared: Prepared, ledger: Ledger, at: Timestamp) -> Result<Accepted, Failure> {
    Ok(match prepared {
        Prepared::Local(Local::Open(title)) => ledger.open(title),
        Prepared::Local(Local::Resolve { escape, task }) => ledger.resolve(escape, task)?,
        Prepared::Local(Local::Historical { escape, reason, correcting }) => {
            ledger.resolve_historical(escape, reason, correcting)?
        }
        Prepared::Local(Local::Defer { task, reason }) => ledger.defer(task, reason)?,
        Prepared::Local(Local::Undefer(task)) => ledger.undefer(task)?,
        Prepared::Local(Local::Role { task, role }) => ledger.role(task, role)?,
        Prepared::Local(Local::Verdict { task, verdict }) => ledger.verdict(task, verdict)?,
        Prepared::Local(Local::Wait { task, question }) => ledger.wait(task, question)?,
        Prepared::Local(Local::Unwait { task, answer }) => ledger.unwait(task, answer)?,
        Prepared::Status(sources) => ledger.status(sources, at),
        Prepared::Claim { task, claimant, origin, rules, allowance } => {
            ledger.claim(task, claimant)?.brief(&origin, &rules, allowance.map(|minutes| at.plus(minutes.span())))
        }
        Prepared::Link { task, pr, pushed, pull } => ledger.link(task, pr)?.pin(&pushed, &pull)?,
        Prepared::Merge { pr, pull } => match ledger.merge(pr)?.clear(&pull)? {
            Clearance::Unmerged(cleared) => cleared.land().record(),
            Clearance::AlreadyMerged(landed) => landed.record(),
        },
        Prepared::Update { updating } => updating.record(),
        Prepared::Retire { task, reason, pull } => ledger.retire(task, reason)?.settle(pull.as_ref())?,
        Prepared::Brief { lease, origin, rules } => ledger.reissue(lease)?.brief(&origin, &rules),
        Prepared::Review { task, agent, pull, rules, allowance } => {
            ledger.review(task, agent)?.brief(&pull, &rules, at.plus(allowance.span()))?
        }
    })
}

/// The per-role lease defaults, read from main.
fn roles<G: Git>(git: &G) -> Result<Roles, Failure> {
    spec(git, SelfFile::Roles, Roles::parse)
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
