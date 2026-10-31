use super::{Answer, Failure, Guard, spec, stamped};
use meta_core::{
    Accepted, Build, LeaseId, Ledger, Record, Rules, SelfFile, Session, Span, Timestamp, Watch, WorkerEnd, Workers,
};
use meta_ports::{Clock, Git, Log, PortError, Sessions};

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

pub(super) fn start<L: Log, G: Git, S: Sessions>(
    lease: LeaseId,
    log: &mut L,
    git: &G,
    sessions: &mut S,
    build: &Build,
    at: Timestamp,
) -> Answer {
    let mut guard = Guard::default();
    if let Err(unverified) = guard.at(log.standing(), git, build) {
        return Answer::failed(Failure::Refused(unverified.into()));
    }
    let place = log.place();
    // Asked before any git or network call, so that a refusal costs nothing.
    let inputs = folded(log.read(), &place).and_then(|ledger| {
        ledger.worker(lease, at)?;
        let rules = spec(git, SelfFile::Rules, Rules::parse)?;
        Ok((limits(log, git)?, git.origin()?, git.binary()?, rules))
    });
    let (workers, origin, binary, rules) = match inputs {
        Ok(inputs) => inputs,
        Err(failure) => return Answer::failed(failure),
    };
    // The session starts inside the lock: the lease and the limit hold until it is recorded, and a
    // session that fails to start leaves the lease unused.
    let outcome = log.transact(|now, records| {
        let decided =
            guard.at(now, git, build).map_err(|unverified| Failure::Refused(unverified.into())).and_then(|()| {
                let ledger = folded(records, &place)?;
                let planned = ledger.worker(lease, at)?.plan(&workers, &origin, &binary, &rules)?;
                let run = sessions.start(planned.session())?;
                Ok(planned.started(run))
            });
        recorded(decided, at)
    });
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
pub(super) fn wait<L: Log, G: Git, S: Sessions, C: Clock>(
    lease: LeaseId,
    log: &mut L,
    git: &G,
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
            return finish(lease, end, log, git, clock.now(), build);
        }
        last = Some(worker);
        clock.pause(POLL);
    }
}

/// Records the end unless another command got there first: `meta stop` records its own, inside the lock.
fn finish<L: Log, G: Git>(
    lease: LeaseId,
    end: WorkerEnd,
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
                match folded(records, &place)?.watch(lease)? {
                    Watch::Running(worker) => Ok(Some(worker.ended(end))),
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
/// and not a session that vanished. A build that may not write the log still kills.
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
        let workers = folded(records, &place).and_then(|ledger| Ok(ledger.stopping(lease)?));
        let workers = match workers {
            Ok(workers) => workers,
            Err(failure) => return (Vec::new(), Err(failure)),
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
    settled(outcome)
}
