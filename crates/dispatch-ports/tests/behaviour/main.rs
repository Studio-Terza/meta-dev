#![deny(clippy::wildcard_enum_match_arm)]

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use proptest::collection::vec;
use proptest::prelude::{any, prop_assert, prop_assert_eq, proptest};
use test_failure::Failed;

use dispatch_ports::{
    Agent, AgentRun, AnsweringShell, Call, Clock, Deadline, Dispatch, DispatchFailure, Ended, Finished,
    HeadlessClaude, Job, MemoryClock, Model, Moment, Output, Pause, Pid, Printed, Prompt, Ran, ReplayedDispatch,
    Reply, Shell, Spoken, Stall, Started, Stderr, SystemClock, SystemShell, Timeout, Tokens, Usage, Wall, Watch,
    Watched, Woke, clock_ops, dispatch_ops, shell_ops,
};
use newtype::Counter;
use port_core::{Released, Run};

#[test]
fn each_envelope_reads_as_its_reply_and_usage_or_its_stall() -> Result<(), Failed> {
    let source = Path::new(file!());
    let table = source.with_file_name(source.file_stem().unwrap_or_default());
    let found = std::env::current_dir()?
        .ancestors()
        .map(|directory| directory.join(&table))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
    let rows: Vec<Vec<String>> = serde_json::from_slice(&fs::read(found)?)?;
    let [envelopes, ..] = rows.as_slice() else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let [fixture, golden, space, newline] = envelopes.as_slice() else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let texts: Vec<String> = serde_json::from_slice(&fs::read(fixture)?)?;
    let read: Vec<String> = texts
        .iter()
        .map(|text| match Watched::from(Output::read(text, Stderr::Silent)) {
            Watched::Finished(finished) => [finished.reply().to_string()]
                .into_iter()
                .chain(finished.usage().spent().iter().map(Tokens::to_string))
                .collect::<Vec<String>>()
                .join(space),
            Watched::Stalled(stall) => stall.to_string(),
        })
        .collect();
    assert_eq!([read.join(newline), newline.clone()].concat(), fs::read_to_string(golden)?);
    Ok(())
}

#[test]
fn each_start_time_is_rendered_as_it_reads_or_is_refused() -> Result<(), Failed> {
    let source = Path::new(file!());
    let table = source.with_file_name(source.file_stem().unwrap_or_default());
    let found = std::env::current_dir()?
        .ancestors()
        .map(|directory| directory.join(&table))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
    let rows: Vec<Vec<String>> = serde_json::from_slice(&fs::read(found)?)?;
    let [_, starts, ..] = rows.as_slice() else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let [fixture, golden, newline] = starts.as_slice() else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let texts: Vec<String> = serde_json::from_slice(&fs::read(fixture)?)?;
    let read: Vec<String> = texts
        .iter()
        .map(|text| match text.parse::<Started>() {
            Ok(started) => started.to_string(),
            Err(unstarted) => unstarted.to_string(),
        })
        .collect();
    let started: Vec<Started> = texts.iter().flat_map(|text| text.parse::<Started>()).collect();
    let reread: Vec<Started> = started.iter().flat_map(|start| start.to_string().parse::<Started>()).collect();
    assert_eq!([read.join(newline), newline.clone()].concat(), fs::read_to_string(golden)?);
    assert_eq!(reread, started);
    Ok(())
}

#[test]
fn a_live_process_group_has_a_start_time_until_it_is_killed() -> Result<(), Failed> {
    let source = Path::new(file!());
    let table = source.with_file_name(source.file_stem().unwrap_or_default());
    let found = std::env::current_dir()?
        .ancestors()
        .map(|directory| directory.join(&table))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
    let rows: Vec<Vec<String>> = serde_json::from_slice(&fs::read(found)?)?;
    let [_, _, sleeper, ..] = rows.as_slice() else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let [program, flag, script, seconds, letter] = sleeper.as_slice() else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let arguments = Vec::leak(vec![&*String::leak(flag.clone()), &*String::leak(script.clone())]);
    let agent = Agent::new(String::leak(program.clone()), arguments);
    let run = Run::root();
    let (before, killed, after, again) = {
        let claude = run.port(HeadlessClaude::new(std::env::current_dir()?, agent));
        let clock = run.port(SystemClock);
        let deadline = Deadline::after(clock.now(), Timeout::new(seconds.parse()?));
        let sleeping = claude.dispatch(&Job::due_by(deadline, Model::Haiku), &Prompt::new(letter.clone()))?;
        let pid = sleeping.pid();
        let before = claude.started(pid)?;
        let killed = claude.kill(pid)?;
        sleeping.end()?;
        (before, killed, claude.started(pid)?, claude.kill(pid)?)
    };
    run.finish().settled()?;
    assert_eq!(before.clone().map(|started| started.to_string().parse::<Started>()).transpose()?, before);
    assert!(before.is_some());
    assert_eq!((killed, after, again), (Ended::Killed, None, Ended::Absent));
    Ok(())
}

#[test]
fn an_agent_that_prints_its_result_is_watched_to_what_the_result_reads_as() -> Result<(), Failed> {
    let source = Path::new(file!());
    let table = source.with_file_name(source.file_stem().unwrap_or_default());
    let found = std::env::current_dir()?
        .ancestors()
        .map(|directory| directory.join(&table))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
    let rows: Vec<Vec<String>> = serde_json::from_slice(&fs::read(found)?)?;
    let [_, _, _, echoing, ..] = rows.as_slice() else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let [program, flag, script, seconds, printed] = echoing.as_slice() else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let arguments = Vec::leak(vec![&*String::leak(flag.clone()), &*String::leak(script.clone())]);
    let agent = Agent::new(String::leak(program.clone()), arguments);
    let run = Run::root();
    let watched = {
        let echo = run.port(HeadlessClaude::new(std::env::current_dir()?, agent));
        let clock = run.port(SystemClock);
        let deadline = Deadline::after(clock.now(), Timeout::new(seconds.parse()?));
        let held = echo.dispatch(&Job::due_by(deadline, Model::Haiku), &Prompt::new(printed.clone()))?;
        Watch::over(&echo, &clock).watched(held)?
    };
    run.finish().settled()?;
    assert_eq!(watched, Watched::from(Output::read(printed, Stderr::Silent)));
    Ok(())
}

#[test]
fn an_agent_is_told_the_model_its_job_names() -> Result<(), Failed> {
    let source = Path::new(file!());
    let table = source.with_file_name(source.file_stem().unwrap_or_default());
    let found = std::env::current_dir()?
        .ancestors()
        .map(|directory| directory.join(&table))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
    let rows: Vec<Vec<String>> = serde_json::from_slice(&fs::read(found)?)?;
    let [_, _, _, _, modelled, ..] = rows.as_slice() else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let [program, flag, script, seconds, golden, newline] = modelled.as_slice() else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let arguments = Vec::leak(vec![&*String::leak(flag.clone()), &*String::leak(script.clone())]);
    let agent = Agent::new(String::leak(program.clone()), arguments);
    let timeout = Timeout::new(seconds.parse()?);
    let directory = std::env::current_dir()?;
    let run = Run::root();
    let watched = [Model::Haiku, Model::Sonnet]
        .into_iter()
        .map(|model| {
            let echo = run.port(HeadlessClaude::new(directory.clone(), agent));
            let clock = run.port(SystemClock);
            let deadline = Deadline::after(clock.now(), timeout);
            let held = echo.dispatch(&Job::due_by(deadline, model), &Prompt::new(String::new()))?;
            Watch::over(&echo, &clock).watched(held)
        })
        .collect::<Result<Vec<Watched>, DispatchFailure>>()?;
    run.finish().settled()?;
    let replies: Vec<String> = watched
        .iter()
        .map(|each| match each {
            Watched::Finished(finished) => finished.reply().to_string(),
            Watched::Stalled(stall) => stall.to_string(),
        })
        .collect();
    assert_eq!([replies.join(newline), newline.clone()].concat(), fs::read_to_string(golden)?);
    Ok(())
}

#[test]
fn an_agent_that_leaves_a_large_prompt_unread_is_killed_and_reaped_at_its_deadline() -> Result<(), Failed> {
    let source = Path::new(file!());
    let table = source.with_file_name(source.file_stem().unwrap_or_default());
    let found = std::env::current_dir()?
        .ancestors()
        .map(|directory| directory.join(&table))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
    let rows: Vec<Vec<String>> = serde_json::from_slice(&fs::read(found)?)?;
    let [_, _, _, _, _, stalling, ..] = rows.as_slice() else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let [shell, dash, sleeping, deaf, seconds, letter, size] = stalling.as_slice() else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let prompt = Prompt::new(letter.repeat(size.parse()?));
    let timeout = Timeout::new(seconds.parse()?);
    let directory = std::env::current_dir()?;
    let program: &'static str = String::leak(shell.clone());
    let flag: &'static str = String::leak(dash.clone());
    let ended = [sleeping, deaf]
        .into_iter()
        .map(|script| {
            let agent = Agent::new(program, Vec::leak(vec![flag, &*String::leak(script.clone())]));
            let run = Run::root();
            let ended = {
                let dispatch = run.port(HeadlessClaude::new(directory.clone(), agent));
                let clock = run.port(MemoryClock::starting(Wall::from(std::time::SystemTime::now())));
                let deadline = Deadline::after(clock.now(), timeout);
                let held = dispatch.dispatch(&Job::due_by(deadline, Model::Haiku), &prompt)?;
                let pid = held.pid();
                let watched = Watch::over(&dispatch, &clock).watched(held)?;
                (watched, dispatch.started(pid)?)
            };
            run.finish().settled().map_err(|failure| io::Error::other(failure.to_string()))?;
            Ok(ended)
        })
        .collect::<Result<Vec<(Watched, Option<Started>)>, Failed>>()?;
    assert_eq!(ended, [(Watched::Stalled(Stall::Overdue), None), (Watched::Stalled(Stall::Overdue), None)]);
    Ok(())
}

#[test]
fn a_wait_nobody_rings_lasts_its_full_pause() -> Result<(), Failed> {
    let source = Path::new(file!());
    let table = source.with_file_name(source.file_stem().unwrap_or_default());
    let found = std::env::current_dir()?
        .ancestors()
        .map(|directory| directory.join(&table))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
    let rows: Vec<Vec<String>> = serde_json::from_slice(&fs::read(found)?)?;
    let [_, _, _, _, _, _, silent, ..] = rows.as_slice() else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let [program, flag, script, seconds, letter, paused] = silent.as_slice() else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let pause = Pause::new(paused.parse()?);
    let start = Wall::from(std::time::SystemTime::now());
    let arguments = Vec::leak(vec![&*String::leak(flag.clone()), &*String::leak(script.clone())]);
    let agent = Agent::new(String::leak(program.clone()), arguments);
    let run = Run::root();
    let waited = {
        let dispatch = run.port(HeadlessClaude::new(std::env::current_dir()?, agent));
        let clock = run.port(MemoryClock::starting(start));
        let deadline = Deadline::after(clock.now(), Timeout::new(seconds.parse()?));
        let silent_run = dispatch.dispatch(&Job::due_by(deadline, Model::Haiku), &Prompt::new(letter.clone()))?;
        let waited = clock.waited(pause, silent_run.bell());
        silent_run.end()?;
        waited
    };
    run.finish().settled()?;
    assert_eq!(waited, start.after(pause));
    Ok(())
}

#[test]
fn a_quiet_bell_waits_out_its_pause() -> Result<(), Failed> {
    let source = Path::new(file!());
    let table = source.with_file_name(source.file_stem().unwrap_or_default());
    let found = std::env::current_dir()?
        .ancestors()
        .map(|directory| directory.join(&table))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
    let rows: Vec<Vec<String>> = serde_json::from_slice(&fs::read(found)?)?;
    let [_, _, _, _, _, _, _, quiet, ..] = rows.as_slice() else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let [millis, seconds] = quiet.as_slice() else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let pause = Duration::from_millis(millis.parse()?);
    let deadline = Deadline::after(Wall::from(std::time::SystemTime::now()), Timeout::new(seconds.parse()?));
    let quiet_run = AgentRun::new(Pid::new(std::process::id()), deadline);
    let started = Instant::now();
    let woke = quiet_run.bell().waited(pause);
    assert_eq!(woke, Woke::Lapsed);
    assert!(started.elapsed() >= pause);
    Ok(())
}

#[test]
fn a_dispatched_agent_rings_its_own_bell_when_it_finishes() -> Result<(), Failed> {
    let source = Path::new(file!());
    let table = source.with_file_name(source.file_stem().unwrap_or_default());
    let found = std::env::current_dir()?
        .ancestors()
        .map(|directory| directory.join(&table))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
    let rows: Vec<Vec<String>> = serde_json::from_slice(&fs::read(found)?)?;
    let [_, _, _, _, _, _, _, _, ringing, ..] = rows.as_slice() else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let [program, flag, script, seconds, letter, waiting] = ringing.as_slice() else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let arguments = Vec::leak(vec![&*String::leak(flag.clone()), &*String::leak(script.clone())]);
    let agent = Agent::new(String::leak(program.clone()), arguments);
    let run = Run::root();
    let woke = {
        let dispatch = run.port(HeadlessClaude::new(std::env::current_dir()?, agent));
        let clock = run.port(MemoryClock::starting(Wall::from(std::time::SystemTime::now())));
        let deadline = Deadline::after(clock.now(), Timeout::new(seconds.parse()?));
        let finished = dispatch.dispatch(&Job::due_by(deadline, Model::Haiku), &Prompt::new(letter.clone()))?;
        let woke = finished.bell().waited(Duration::from_secs(waiting.parse()?));
        finished.end()?;
        woke
    };
    run.finish().settled()?;
    assert_eq!(woke, Woke::Rung);
    Ok(())
}

#[test]
fn a_shell_call_returns_its_printing_and_whether_it_succeeded_failed_or_ran_past_its_deadline() -> Result<(), Failed> {
    let source = Path::new(file!());
    let table = source.with_file_name(source.file_stem().unwrap_or_default());
    let found = std::env::current_dir()?
        .ancestors()
        .map(|directory| directory.join(&table))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
    let rows: Vec<Vec<String>> = serde_json::from_slice(&fs::read(found)?)?;
    let [_, _, _, _, _, _, _, _, _, shelled, ..] = rows.as_slice() else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let [shell_program, flag, succeeding, failing, refused, sleeping, seconds, overdue, newline] = shelled.as_slice()
    else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let directory = std::env::current_dir()?;
    let program: &'static str = String::leak(shell_program.clone());
    let run = Run::root();
    let ran = {
        let shell = run.port(SystemShell);
        let clock = run.port(SystemClock);
        [(succeeding, seconds), (failing, seconds), (sleeping, overdue)]
            .into_iter()
            .map(|(script, timeout)| {
                let deadline = Deadline::after(clock.now(), Timeout::new(timeout.parse()?));
                let call = Call::new(program, vec![flag.clone(), script.clone()], directory.clone(), deadline);
                Ok(shell.ran(&call)?)
            })
            .collect::<Result<Vec<Ran>, Failed>>()?
    };
    let calls = run.finish().settled()?.calls::<shell_ops::Ran>();
    let printed = [directory.to_string_lossy().into_owned(), newline.clone()].concat();
    assert_eq!(ran, [Ran::Succeeded(Printed::new(printed)), Ran::Failed(Printed::new(refused.clone())), Ran::Overdue]);
    assert_eq!(calls.count(), ran.len());
    Ok(())
}

proptest! {
    #[test]
    fn a_run_that_never_replies_is_watched_until_its_deadline_passes_then_released_once(
        start in any::<i32>(),
        seconds in 0..120_u32,
    ) {
        let replayed = ReplayedDispatch::replaying([None]);
        let run = Run::root();
        let (watched, now, deadline) = {
            let dispatch = run.port(&replayed);
            let clock = run.port(MemoryClock::starting(Wall::new(i64::from(start))));
            let deadline = Deadline::after(clock.now(), Timeout::new(seconds));
            let held = dispatch.dispatch(&Job::due_by(deadline, Model::Haiku), &Prompt::new(String::new()))?;
            (Watch::over(&dispatch, &clock).watched(held)?, clock.now(), deadline)
        };
        let settled = run.finish().settled()?;
        prop_assert_eq!(watched, Watched::Stalled(Stall::Overdue));
        prop_assert!(Deadline::after(now, Timeout::new(u32::default())) >= deadline);
        prop_assert_eq!(settled.calls::<clock_ops::Waited>(), settled.calls::<dispatch_ops::Output>());
        prop_assert_eq!(settled.calls::<Released<AgentRun>>(), settled.calls::<dispatch_ops::Dispatch>());
    }

    #[test]
    fn a_run_that_replies_is_finished_on_its_first_poll_and_released_once(
        reply in any::<Reply>(),
        spent in any::<[Tokens; 4]>(),
        seconds in 1..120_u32,
    ) {
        let [input, output, written, read] = spent;
        let finished = Finished::new(reply, Usage::new(input, output, written, read));
        let replayed = ReplayedDispatch::replaying([Some(Output::Finished(finished.clone()))]);
        let run = Run::root();
        let watched = {
            let dispatch = run.port(&replayed);
            let clock = run.port(MemoryClock::starting(Wall::new(i64::default())));
            let deadline = Deadline::after(clock.now(), Timeout::new(seconds));
            let held = dispatch.dispatch(&Job::due_by(deadline, Model::Sonnet), &Prompt::new(String::new()))?;
            Watch::over(&dispatch, &clock).watched(held)?
        };
        let settled = run.finish().settled()?;
        prop_assert_eq!(watched, Watched::Finished(finished));
        prop_assert_eq!(settled.calls::<clock_ops::Waited>().count(), usize::default());
        prop_assert_eq!(settled.calls::<dispatch_ops::Output>(), settled.calls::<dispatch_ops::Dispatch>());
        prop_assert_eq!(settled.calls::<Released<AgentRun>>(), settled.calls::<dispatch_ops::Dispatch>());
    }

    #[test]
    fn a_fanned_job_hands_each_prompt_to_its_own_run_and_watches_each_to_its_reply(
        prompts in vec(any::<Prompt>(), 0..6),
        replies in vec(any::<Reply>(), 0..6),
    ) {
        let outputs: Vec<Output> =
            replies.iter().map(|reply| Output::Finished(Finished::new(reply.clone(), Usage::default()))).collect();
        let replayed = ReplayedDispatch::replaying(outputs.iter().cloned().map(Some));
        let run = Run::root();
        let watched = {
            let dispatch = run.port(&replayed);
            let clock = run.port(MemoryClock::starting(Wall::new(i64::default())));
            let deadline = Deadline::after(clock.now(), Timeout::new(u32::MAX));
            let fanned = prompts.len().min(outputs.len());
            Watch::over(&dispatch, &clock).fanned(&Job::due_by(deadline, Model::Haiku), prompts.get(..fanned).unwrap_or_default())?
        };
        run.finish().settled()?;
        let fanned = prompts.len().min(outputs.len());
        prop_assert_eq!(watched, outputs.into_iter().take(fanned).map(Watched::from).collect::<Vec<Watched>>());
        prop_assert_eq!(replayed.prompts(), prompts.into_iter().take(fanned).collect::<Vec<Prompt>>());
    }

    #[test]
    fn an_answering_shell_answers_the_calls_it_holds_and_refuses_the_rest(
        arguments in vec("[a-z]{1,8}", 0..4),
        unknown in vec("[A-Z]{1,8}", 1..4),
        printed in ".*",
        spelled in "[a-z]{1,8}",
    ) {
        let program: &'static str = String::leak(spelled);
        let deadline = Deadline::after(Wall::new(i64::default()), Timeout::new(u32::default()));
        let held = Call::new(program, arguments, PathBuf::new(), deadline);
        let other = Call::new(program, unknown, PathBuf::new(), deadline);
        let answer = Ran::Succeeded(Printed::new(printed));
        let shell = AnsweringShell::answering(BTreeMap::from([(Spoken::from(&held), answer.clone())]));
        let run = Run::root();
        let (answered, refused) = {
            let port = run.port(&shell);
            (port.ran(&held)?, port.ran(&other))
        };
        prop_assert_eq!(answered, answer);
        prop_assert!(matches!(refused, Err(DispatchFailure::Unanswered)));
        prop_assert_eq!(shell.asked(), [held, other]);
    }

    #[test]
    fn a_pause_lasts_its_whole_seconds(seconds in any::<u32>()) {
        prop_assert_eq!(Duration::from(Pause::new(seconds)), Duration::from_secs(u64::from(seconds)));
    }

    #[test]
    fn a_moment_counts_the_milliseconds_between_two_walls(start in any::<i32>(), seconds in any::<u32>()) {
        let early = Wall::new(i64::from(start));
        let late = early.after(Pause::new(seconds));
        let [first, last] = [early, late].map(Moment::from);
        prop_assert_eq!(i128::from(last.since(first)), i128::try_from(Duration::from(Pause::new(seconds)).as_millis())?);
        prop_assert_eq!(first.since(last), last.since(first).saturating_neg());
        prop_assert_eq!(last.since(last), i64::default());
    }
}
