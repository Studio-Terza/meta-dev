use super::{Answer, Failure, Guard, pull_at, spec, stamped};
use meta_core::{
    Accepted, AgentId, Build, ClaudeVersion, Judged, LeaseId, Ledger, NoVerdict, Record, Rules, SelfFile, Session,
    SessionVerdict, Span, Timestamp, Watch, Worker, WorkerEnd, Workers,
};
use meta_ports::{Clock, Git, GitHub, Log, PortError, Sessions};

/// How often a supervisor looks at its worker, and so how late it can be at the deadline.
const POLL: Span = Span::of_seconds(5);

fn folded(records: Result<Vec<Record>, PortError>, place: &str) -> Result<Ledger, Failure> {
    Ledger::fold(&records?).map_err(|corrupt| Failure::Broken(corrupt.in_log(place)))
}

/// `self/workers.toml`: main's copy for the real log, the working tree's for a scratch log, so that
/// a branch can run the file it changes without its allowlist ever reaching the real log.
fn limits<L: Log, G: Git>(log: &L, git: &G) -> Result<Workers, Failure> {
    match log.standing().is_real() {
        true => spec(git, SelfFile::Workers, Workers::parse),
        false => Ok(Workers::parse(&git.local(SelfFile::Workers)?)?),
    }
}

pub(super) fn start<L: Log, G: Git, H: GitHub, S: Sessions, C: Clock>(
    lease: LeaseId,
    log: &mut L,
    git: &G,
    github: &H,
    sessions: &mut S,
    clock: &C,
    build: &Build,
) -> Answer {
    let mut guard = Guard::default();
    if let Err(unverified) = guard.at(log.standing(), git, build) {
        return Answer::failed(Failure::Refused(unverified.into()));
    }
    let at = clock.now();
    let place = log.place();
    // Asked before any git or network call, so that a refusal costs nothing.
    let inputs = folded(log.read(), &place).and_then(|ledger| {
        let starting = ledger.worker(lease, at)?;
        let rules = spec(git, SelfFile::Rules, Rules::parse)?;
        let workers = limits(log, git)?;
        starting.within_limit(&workers)?;
        workers.admit(ClaudeVersion::from_reply(&sessions.version()?).map_err(PortError::from)?)?;
        // The diff is read before the head is checked, so a head that moved meanwhile refuses.
        let copy = github.review_copy(starting.pr())?;
        starting.at_head(&github.pull(starting.pr())?)?;
        let (origin, binary) = (git.origin()?, git.binary()?);
        // Fetched by the lease's own commit, which cannot move, and outside the lock, so that a slow
        // fetch holds up neither the log nor `meta stop`.
        let staged = sessions.stage(&starting.worker(), &origin, starting.head()).map_err(|error| {
            PortError::new(format!(
                "no checkout of {} for {lease}: {error}\nNothing started and {lease} stays unused. Next: meta worker start {lease} tries again.",
                starting.head()
            ))
        })?;
        Ok((workers, copy, staged, origin, binary, rules))
    });
    let (workers, copy, staged, origin, binary, rules) = match inputs {
        Ok(inputs) => inputs,
        Err(failure) => return Answer::failed(failure),
    };
    // Taken after the checkout is staged, so that a slow fetch does not come off the worker's deadline.
    let at = clock.now();
    // The session starts inside the lock: the lease and the limit hold until it is recorded, and a
    // session that fails to start leaves the lease unused.
    let outcome = log.transact(|now, records| {
        let decided =
            guard.at(now, git, build).map_err(|unverified| Failure::Refused(unverified.into())).and_then(|()| {
                let ledger = folded(records, &place)?;
                let staged = staged.clone();
                let planned = ledger.worker(lease, at)?.plan(&workers, copy, staged, &origin, &binary, &rules)?;
                let run = sessions.start(planned.session())?;
                Ok(planned.started(run))
            });
        recorded(decided, at)
    });
    sessions.unstage(&staged);
    settled(outcome)
}

fn recorded(decided: Result<Accepted, Failure>, at: Timestamp) -> (Vec<Record>, Result<String, Failure>) {
    match decided.map(Accepted::into_parts) {
        Ok((events, text)) => (stamped(events, at), Ok(text)),
        Err(failure) => (Vec::new(), Err(failure)),
    }
}

fn settled(outcome: Result<Result<String, Failure>, PortError>) -> Answer {
    match outcome.map_err(Failure::from).and_then(|decided| decided) {
        Ok(text) => Answer::done(text),
        Err(failure) => Answer::failed(failure),
    }
}

/// Watches the worker on `lease` until it ends or its deadline passes, then records the end. The
/// log is read through its lock, so the first look waits for `start` to record the worker. A read
/// that fails after that keeps the last look: the deadline holds even on a broken log. When this
/// process ends for any reason, the session ends with it (`Sessions::start`).
pub(super) fn wait<L: Log, G: Git, H: GitHub, S: Sessions, C: Clock>(
    lease: LeaseId,
    log: &mut L,
    git: &G,
    github: &H,
    sessions: &mut S,
    clock: &C,
    build: &Build,
) -> Answer {
    let place = log.place();
    let mut last = None;
    loop {
        let look = log
            .transact(|_, records| (Vec::new(), folded(records, &place).and_then(|ledger| Ok(ledger.watch(lease)?))))
            .map_err(Failure::from)
            .and_then(|looked| looked);
        let worker = match (look, last.take()) {
            (Ok(Watch::Running(worker)), _) | (Err(_), Some(worker)) => worker,
            (Ok(Watch::Over), _) => return Answer::done(format!("{lease}: the worker's end is already recorded\n")),
            (Err(failure), None) => return Answer::failed(failure),
        };
        let end = match sessions.state(worker.agent(), worker.run()) {
            Ok(Session::Ended(status)) => Some(WorkerEnd::from(status)),
            _ if worker.is_past_deadline(clock.now()) => sessions.stop(worker.run()).ok().map(|()| WorkerEnd::Deadline),
            _ => None,
        };
        if let Some(end) = end {
            let judged = judged(&worker, end, github, sessions, clock);
            return finish(lease, (end, judged), log, git, clock.now(), build);
        }
        last = Some(worker);
        clock.pause(POLL);
    }
}

/// What the ended session leaves: the verdict in its transcript, and the PR as GitHub reports it
/// now, which ties the verdict to the head the worker reviewed. Asked outside the lock.
fn judged<H: GitHub, S: Sessions, C: Clock>(
    worker: &Worker,
    end: WorkerEnd,
    github: &H,
    sessions: &S,
    clock: &C,
) -> Judged {
    end.no_verdict().map_or(Ok(()), Err)?;
    let lines = sessions.transcript(worker.run()).map_err(NoVerdict::because)?;
    let verdict = SessionVerdict::from_transcript(&lines)?;
    let pull = pull_at(github, clock, worker.pr(), worker.head()).map_err(NoVerdict::because)?;
    Ok((verdict, pull))
}

/// Records the end and any verdict unless another command got there first: `meta stop` records its
/// own, inside the lock.
fn finish<L: Log, G: Git>(
    lease: LeaseId,
    (end, judged): (WorkerEnd, Judged),
    log: &mut L,
    git: &G,
    at: Timestamp,
    build: &Build,
) -> Answer {
    let mut guard = Guard::default();
    let place = log.place();
    let outcome = log.transact(|now, records| {
        let decided =
            guard.at(now, git, build).map_err(|unverified| Failure::Refused(unverified.into())).and_then(|()| {
                let ledger = folded(records, &place)?;
                match ledger.watch(lease)? {
                    Watch::Running(worker) => Ok(Some(ledger.worker_ended(worker, end, judged))),
                    Watch::Over => Ok(None),
                }
            });
        match decided {
            Ok(Some(ended)) => recorded(Ok(ended), at),
            Ok(None) => (Vec::new(), Ok(format!("{lease}: the worker's end is already recorded\n"))),
            Err(failure) => (Vec::new(), Err(failure)),
        }
    });
    settled(outcome)
}

/// The kill switch. Each group ends inside the lock, so that its supervisor finds the end recorded
/// and not a session that vanished. A build that may not write the log still kills, and so does a
/// log that cannot be read: then the workers come from their own directories (`Sessions::listed`).
pub(super) fn stop<L: Log, G: Git, S: Sessions>(
    lease: Option<LeaseId>,
    log: &mut L,
    git: &G,
    sessions: &mut S,
    build: &Build,
    at: Timestamp,
) -> Answer {
    let mut guard = Guard::default();
    let place = log.place();
    let outcome = log.transact(|now, records| {
        let ledger = match folded(records, &place) {
            Ok(ledger) => ledger,
            Err(failure) => return (Vec::new(), Err(stop_listed(lease, sessions, &failure))),
        };
        let workers = match ledger.stopping(lease) {
            Ok(workers) => workers,
            Err(refusal) => return (Vec::new(), Err(refusal.into())),
        };
        if workers.is_empty() {
            return (Vec::new(), Ok("no workers are running\n".to_owned()));
        }
        let verified = guard.at(now, git, build);
        let mut events = Vec::new();
        let mut text = String::new();
        let mut failed = Vec::new();
        for worker in workers {
            let id = worker.agent().clone();
            match sessions.stop(worker.run()) {
                Ok(()) => {
                    let (ended, line) = worker.ended(WorkerEnd::Stopped).into_parts();
                    events.extend(ended);
                    text.push_str(&line);
                }
                Err(error) => failed.push(format!("{id}: {error}")),
            }
        }
        if let Err(unverified) = verified {
            text.push_str(&format!("not recorded: {unverified}\n"));
            events.clear();
        }
        let decided = match failed.is_empty() {
            true => Ok(text),
            false => Err(Failure::Broken(format!("{text}could not stop {}", failed.join("; ")))),
        };
        (stamped(events, at), decided)
    });
    match outcome {
        Ok(decided) => settled(Ok(decided)),
        Err(error) => Answer::failed(stop_listed(lease, sessions, &Failure::from(error))),
    }
}

/// Kills the workers found without the log, which cannot record it: the failure says what became
/// of each, after `why` the log could not be used.
fn stop_listed<S: Sessions>(lease: Option<LeaseId>, sessions: &mut S, why: &Failure) -> Failure {
    let wanted = lease.map(AgentId::worker);
    let mut killed = Vec::new();
    let mut failed = Vec::new();
    let listed = sessions.listed().unwrap_or_else(|error| vec![Err(error)]);
    for entry in listed {
        match entry {
            Ok((id, run)) if wanted.as_ref().is_none_or(|wanted| *wanted == id) => match sessions.stop(&run) {
                Ok(()) => killed.push(id.to_string()),
                Err(error) => failed.push(format!("{id}: {error}")),
            },
            Ok(_) => {}
            Err(error) => failed.push(error.to_string()),
        }
    }
    let killed = match killed.is_empty() {
        true => "no worker found to kill".to_owned(),
        false => format!("killed {}", killed.join(", ")),
    };
    let failed = match failed.is_empty() {
        true => String::new(),
        false => format!("; could not stop {}", failed.join("; ")),
    };
    Failure::Broken(format!("{why}\n{killed}{failed}; nothing was recorded, for the log could not be used"))
}
