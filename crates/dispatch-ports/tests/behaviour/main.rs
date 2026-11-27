#![deny(clippy::wildcard_enum_match_arm)]

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use proptest::char::range;
use proptest::collection::vec;
use proptest::prelude::{any, prop_assert, prop_assert_eq, proptest};
use proptest::test_runner::Config;
use serde::Deserialize;
use serde::de::IgnoredAny;
use test_failure::Failed;

use dispatch_ports::{
    AgentRun, AnsweringShell, Call, Clock, Deadline, Dispatch, DispatchFailure, Ended, Finished, HeadlessClaude, Job,
    MemoryClock, Model, Moment, Output, Pause, Pid, Printed, Program, Prompt, Ran, ReplayedDispatch, Reply, Shell,
    Spoken, Stall, Started, Stderr, SystemClock, SystemShell, Timeout, Tokens, Usage, Wall, Watch, Watched, Woke,
    clock_ops, dispatch_ops, shell_ops,
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
    let bytes = fs::read(found)?;
    let mut rows = serde_json::Deserializer::from_slice(&bytes);
    let (fixture, golden) = <(PathBuf, PathBuf)>::deserialize(&mut rows)?;
    let texts: Vec<String> = serde_json::from_slice(&fs::read(fixture)?)?;
    let read: Vec<Result<(Reply, Vec<Tokens>), String>> = texts
        .iter()
        .map(|text| match Watched::from(Output::read(text, Stderr::Silent)) {
            Watched::Finished(finished) => Ok((finished.reply().clone(), finished.usage().spent().to_vec())),
            Watched::Stalled(stall) => Err(stall.to_string()),
        })
        .collect();
    assert_eq!(read, serde_json::from_slice::<Vec<_>>(&fs::read(golden)?)?);
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
    let bytes = fs::read(found)?;
    let mut rows = serde_json::Deserializer::from_slice(&bytes);
    IgnoredAny::deserialize(&mut rows)?;
    let (fixture, golden) = <(PathBuf, PathBuf)>::deserialize(&mut rows)?;
    let texts: Vec<String> = serde_json::from_slice(&fs::read(fixture)?)?;
    let read: Vec<Result<Started, String>> =
        texts.iter().map(|text| text.parse::<Started>().map_err(|refused| refused.to_string())).collect();
    let rendered: Vec<Result<String, String>> =
        read.iter().map(|started| started.as_ref().map(ToString::to_string).map_err(Clone::clone)).collect();
    let reread: Vec<Started> = read.iter().flatten().flat_map(|start| start.to_string().parse::<Started>()).collect();
    assert_eq!((read.clone(), rendered), serde_json::from_slice(&fs::read(golden)?)?);
    assert_eq!(reread, read.into_iter().flatten().collect::<Vec<Started>>());
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
    let bytes = fs::read(found)?;
    let mut rows = serde_json::Deserializer::from_slice(&bytes);
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    let (program, timeout, prompt) = <(Program, Timeout, Prompt)>::deserialize(&mut rows)?;
    let run = Run::root();
    let (before, killed, after, again) = {
        let claude = run.port(HeadlessClaude::new(std::env::current_dir()?, program));
        let clock = run.port(SystemClock);
        let sleeping = claude.dispatch(&Job::due_by(Deadline::after(clock.now(), timeout), Model::Haiku), &prompt)?;
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
    let bytes = fs::read(found)?;
    let mut rows = serde_json::Deserializer::from_slice(&bytes);
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    let (program, timeout, printed) = <(Program, Timeout, Prompt)>::deserialize(&mut rows)?;
    let run = Run::root();
    let watched = {
        let echo = run.port(HeadlessClaude::new(std::env::current_dir()?, program));
        let clock = run.port(SystemClock);
        let held = echo.dispatch(&Job::due_by(Deadline::after(clock.now(), timeout), Model::Haiku), &printed)?;
        Watch::over(&echo, &clock).watched(held)?
    };
    run.finish().settled()?;
    let envelope = String::from_utf8(printed.as_ref().to_vec())?;
    assert_eq!(watched, Watched::from(Output::read(&envelope, Stderr::Silent)));
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
    let bytes = fs::read(found)?;
    let mut rows = serde_json::Deserializer::from_slice(&bytes);
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    let (program, timeout, golden) = <(Program, Timeout, PathBuf)>::deserialize(&mut rows)?;
    let directory = std::env::current_dir()?;
    let run = Run::root();
    let replies = [Model::Haiku, Model::Sonnet]
        .into_iter()
        .map(|model| {
            let echo = run.port(HeadlessClaude::new(directory.clone(), program.clone()));
            let clock = run.port(SystemClock);
            let held =
                echo.dispatch(&Job::due_by(Deadline::after(clock.now(), timeout), model), &Prompt::new(String::new()))?;
            Watch::over(&echo, &clock).watched(held).map(|watched| match watched {
                Watched::Finished(finished) => Ok(finished.reply().clone()),
                Watched::Stalled(stall) => Err(stall.to_string()),
            })
        })
        .collect::<Result<Vec<Result<Reply, String>>, DispatchFailure>>()?;
    run.finish().settled()?;
    assert_eq!(replies, serde_json::from_slice::<Vec<_>>(&fs::read(golden)?)?);
    Ok(())
}

proptest! {
    #![proptest_config(Config { cases: 2, ..Config::default() })]
    #[test]
    fn an_agent_that_leaves_a_large_prompt_unread_is_killed_and_reaped_at_its_deadline(
        letters in vec(range('a', 'z'), 0x0001_0000..0x0002_0000),
    ) {
        let source = Path::new(file!());
        let table = source.with_file_name(source.file_stem().unwrap_or_default());
        let found = std::env::current_dir()?
            .ancestors()
            .map(|directory| directory.join(&table))
            .find(|candidate| candidate.is_file())
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
        let bytes = fs::read(found)?;
        let mut rows = serde_json::Deserializer::from_slice(&bytes);
        IgnoredAny::deserialize(&mut rows)?;
        IgnoredAny::deserialize(&mut rows)?;
        IgnoredAny::deserialize(&mut rows)?;
        IgnoredAny::deserialize(&mut rows)?;
        IgnoredAny::deserialize(&mut rows)?;
        let (programs, timeout) = <(Vec<Program>, Timeout)>::deserialize(&mut rows)?;
        let prompt = Prompt::new(letters.into_iter().collect());
        let directory = std::env::current_dir()?;
        for program in programs {
            let run = Run::root();
            let ended = {
                let dispatch = run.port(HeadlessClaude::new(directory.clone(), program));
                let clock = run.port(MemoryClock::starting(Wall::from(SystemTime::now())));
                let deadline = Deadline::after(clock.now(), timeout);
                let held = dispatch.dispatch(&Job::due_by(deadline, Model::Haiku), &prompt)?;
                let pid = held.pid();
                let watched = Watch::over(&dispatch, &clock).watched(held)?;
                (watched, dispatch.started(pid)?)
            };
            run.finish().settled()?;
            prop_assert_eq!(ended, (Watched::Stalled(Stall::Overdue), None));
        }
    }
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
    let bytes = fs::read(found)?;
    let mut rows = serde_json::Deserializer::from_slice(&bytes);
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    let (program, timeout, prompt, pause) = <(Program, Timeout, Prompt, Pause)>::deserialize(&mut rows)?;
    let start = Wall::from(SystemTime::now());
    let run = Run::root();
    let waited = {
        let dispatch = run.port(HeadlessClaude::new(std::env::current_dir()?, program));
        let clock = run.port(MemoryClock::starting(start));
        let silent = dispatch.dispatch(&Job::due_by(Deadline::after(clock.now(), timeout), Model::Haiku), &prompt)?;
        let waited = clock.waited(pause, silent.bell());
        silent.end()?;
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
    let bytes = fs::read(found)?;
    let mut rows = serde_json::Deserializer::from_slice(&bytes);
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    let (pause, timeout) = <(Pause, Timeout)>::deserialize(&mut rows)?;
    let deadline = Deadline::after(Wall::from(SystemTime::now()), timeout);
    let quiet = AgentRun::new(Pid::new(std::process::id()), deadline);
    let started = Instant::now();
    let woke = quiet.bell().waited(Duration::from(pause));
    assert_eq!(woke, Woke::Lapsed);
    assert!(started.elapsed() >= Duration::from(pause));
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
    let bytes = fs::read(found)?;
    let mut rows = serde_json::Deserializer::from_slice(&bytes);
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    let (program, timeout, prompt, waiting) = <(Program, Timeout, Prompt, Pause)>::deserialize(&mut rows)?;
    let run = Run::root();
    let woke = {
        let dispatch = run.port(HeadlessClaude::new(std::env::current_dir()?, program));
        let clock = run.port(MemoryClock::starting(Wall::from(SystemTime::now())));
        let finished = dispatch.dispatch(&Job::due_by(Deadline::after(clock.now(), timeout), Model::Haiku), &prompt)?;
        let woke = finished.bell().waited(Duration::from(waiting));
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
    let bytes = fs::read(found)?;
    let mut rows = serde_json::Deserializer::from_slice(&bytes);
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    let calls = Vec::<(Program, Timeout, Ran)>::deserialize(&mut rows)?;
    let directory = std::env::current_dir()?;
    let run = Run::root();
    let ran = {
        let shell = run.port(SystemShell);
        let clock = run.port(SystemClock);
        calls
            .iter()
            .map(|(program, timeout, _)| {
                let call = Call::new(program.clone(), directory.clone(), Deadline::after(clock.now(), *timeout));
                shell.ran(&call)
            })
            .collect::<Result<Vec<Ran>, DispatchFailure>>()?
    };
    let counted = run.finish().settled()?.calls::<shell_ops::Ran>();
    assert_eq!(ran, calls.iter().map(|(_, _, expected)| expected.clone()).collect::<Vec<Ran>>());
    assert_eq!(counted.count(), ran.len());
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
        spent in vec(any::<Tokens>(), 4),
        seconds in 1..120_u32,
    ) {
        let usage = match spent.as_slice() {
            [input, output, written, read] => Usage::new(*input, *output, *written, *read),
            _ => Usage::default(),
        };
        let finished = Finished::new(reply, usage);
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
        let fanned = prompts.len().min(outputs.len());
        let run = Run::root();
        let watched = {
            let dispatch = run.port(&replayed);
            let clock = run.port(MemoryClock::starting(Wall::new(i64::default())));
            let job = Job::due_by(Deadline::after(clock.now(), Timeout::new(u32::MAX)), Model::Haiku);
            Watch::over(&dispatch, &clock).fanned(&job, prompts.get(..fanned).unwrap_or_default())?
        };
        run.finish().settled()?;
        prop_assert_eq!(watched, outputs.into_iter().take(fanned).map(Watched::from).collect::<Vec<Watched>>());
        prop_assert_eq!(replayed.prompts(), prompts.into_iter().take(fanned).collect::<Vec<Prompt>>());
    }

    #[test]
    fn an_answering_shell_answers_the_calls_it_holds_and_refuses_the_rest(
        arguments in vec("[a-z]{1,8}", 0..4),
        unknown in vec("[A-Z]{1,8}", 1..4),
        printed in ".*",
        name in "[a-z]{1,8}",
    ) {
        let deadline = Deadline::after(Wall::new(i64::default()), Timeout::new(u32::default()));
        let held = Call::new(Program::new(&name, arguments), PathBuf::new(), deadline);
        let other = Call::new(Program::new(&name, unknown), PathBuf::new(), deadline);
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
