use std::os::unix::process::CommandExt;
use std::process::Command;
use std::time::{Duration, Instant};

use dispatch_ports::{
    Agent, AgentRun, Call, Clock, Count, Deadline, Dispatch, HeadlessClaude, Job, Model, Output, Oversleep, Pause, Pid,
    Prompt, Reply, Ringing, Scripted, Shell, Started, Stderr, Stepped, SystemClock, SystemShell, Timeout, Wall, Watch,
    clock_ops, dispatch_ops, shell_ops,
};
use fixture::{Cases, Debugged, Failed, Fixture, Spelled};
use invariants::Ignoring;
use newtype::Wrapping;
use port_core::{Released, Run};
use serde::Deserialize;

use super::ignored::Ignored;

#[derive(Fixture, Deserialize, Debug)]
struct Envelopes {
    texts: Vec<Spelled>,
}

#[derive(Fixture, Deserialize, Debug)]
struct Starts {
    texts: Vec<Spelled>,
}

#[derive(Fixture, Deserialize, Debug)]
struct Watched {
    start: Wall,
    timeouts: Vec<Timeout>,
    paces: Vec<Paced>,
}

#[derive(Deserialize, Debug)]
struct Paced {
    #[serde(default)]
    oversleep: Option<Oversleep>,
    pid: Pid,
    reply: Reply,
    #[serde(default)]
    finishing: Option<Count>,
    #[serde(default)]
    ringing: Ringing,
}

#[derive(Fixture, Deserialize, Debug)]
struct Living {
    program: Spelled,
    arguments: Vec<Spelled>,
}

#[derive(Fixture, Deserialize, Debug)]
struct Agents {
    agent: Agent,
    prompt: Spelled,
    models: Vec<Model>,
    timeout: Timeout,
}

#[derive(Fixture, Deserialize, Debug)]
struct Stalling {
    agents: Vec<Agent>,
    filler: Spelled,
    size: Count,
    start: Wall,
    timeout: Timeout,
}

#[derive(Fixture, Deserialize, Debug)]
struct Bells {
    sleeping: Agent,
    speaking: Agent,
    prompt: Spelled,
    start: Wall,
    pause: Pause,
    quiet: Duration,
    timeout: Timeout,
}

#[derive(Fixture, Deserialize, Debug)]
struct Shells {
    program: Spelled,
    calls: Vec<Called>,
}

#[derive(Deserialize, Debug)]
struct Called {
    arguments: Vec<Spelled>,
    timeout: Timeout,
}

#[test]
fn every_agent_result_and_start_time_reads_as_its_golden_says() -> Result<(), Failed> {
    let envelopes = Cases::<Envelopes>::drifted(|case| {
        let read: Vec<_> = case.row().texts.iter().map(|text| Output::read(text.as_str(), Stderr::Silent)).collect();
        Ok(Debugged(read).to_string())
    })?;
    let starts = Cases::<Starts>::drifted(|case| {
        let read: Vec<_> = case
            .row()
            .texts
            .iter()
            .map(|text| {
                let started = text.as_str().parse::<Started>().ignoring(Ignored::Unparsed);
                let again =
                    started.as_ref().map(|read| read.to_string().parse::<Started>().ignoring(Ignored::Unparsed));
                (text, started, again)
            })
            .collect();
        Ok(Debugged(read).to_string())
    })?;
    assert_eq!((envelopes, starts), (Vec::new(), Vec::new()));
    Ok(())
}

#[test]
fn every_watch_over_the_stepped_clock_ends_as_its_golden_says() -> Result<(), Failed> {
    let drifted = Cases::<Watched>::drifted(|case| {
        let Watched { start, timeouts, paces } = case.into_row();
        let observed = timeouts
            .iter()
            .flat_map(|timeout| paces.iter().map(move |paced| (*timeout, paced)))
            .map(|(timeout, paced)| {
                let stepped = Stepped::starting(start, paced.oversleep);
                let ports = Run::root();
                let watched = {
                    let clock = ports.port(stepped.clone());
                    let scripted = Scripted::replying(paced.pid, paced.reply.clone(), paced.finishing, paced.ringing);
                    let dispatch = ports.port(scripted);
                    let deadline = Deadline::after(start, timeout);
                    let held = dispatch.dispatch(&Job::due_by(deadline, Model::Haiku), &Prompt::from(&paced.reply))?;
                    Watch::over(&dispatch, &clock).watched(held)?
                };
                let snapshot = ports.finish().settled()?;
                let calls = [
                    snapshot.calls::<dispatch_ops::Output>(),
                    snapshot.calls::<clock_ops::Waited>(),
                    snapshot.calls::<Released<AgentRun>>(),
                ];
                Ok((timeout, watched, calls, stepped.waits()))
            })
            .collect::<Result<Vec<_>, Failed>>()?;
        Ok(Debugged(observed).to_string())
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}

#[test]
fn a_live_process_group_has_a_start_time_until_it_is_killed() -> Result<(), Failed> {
    let drifted = Cases::<Living>::drifted(|case| {
        let directory = case.directory().to_path_buf();
        let Living { program, arguments } = case.into_row();
        let mut sleeper =
            Command::new(program.as_str()).args(arguments.iter().map(Spelled::as_str)).process_group(0).spawn()?;
        let pid = Pid::new(sleeper.id());
        let ports = Run::root();
        let claude = ports.port(HeadlessClaude::new(directory, Agent::claude()));
        let before = claude.started(pid)?.is_some();
        let killed = claude.kill(pid)?;
        sleeper.wait()?;
        let after = claude.started(pid)?;
        let again = claude.kill(pid)?;
        Ok(Debugged((before, killed, after, again)).to_string())
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}

#[test]
fn an_agent_is_watched_to_the_reply_and_usage_it_prints_for_the_model_its_job_names() -> Result<(), Failed> {
    let drifted = Cases::<Agents>::drifted(|case| {
        let directory = case.directory().to_path_buf();
        let Agents { agent, prompt, models, timeout } = case.into_row();
        let watched = models
            .iter()
            .map(|model| {
                let ports = Run::root();
                let watched = {
                    let echo = ports.port(HeadlessClaude::new(directory.clone(), agent.clone()));
                    let clock = ports.port(SystemClock);
                    let deadline = Deadline::after(clock.now(), timeout);
                    let held = echo.dispatch(&Job::due_by(deadline, *model), &Prompt::from(&prompt))?;
                    Watch::over(&echo, &clock).watched(held)?
                };
                ports.finish().settled()?;
                Ok((model, watched))
            })
            .collect::<Result<Vec<_>, Failed>>()?;
        Ok(Debugged(watched).to_string())
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}

#[test]
fn an_agent_that_leaves_a_large_prompt_unread_is_killed_and_reaped_at_its_deadline() -> Result<(), Failed> {
    let drifted = Cases::<Stalling>::drifted(|case| {
        let directory = case.directory().to_path_buf();
        let Stalling { agents, filler, size, start, timeout } = case.into_row();
        let repeated = usize::try_from(size.into_inner())?;
        let stalled = agents
            .iter()
            .map(|agent| {
                let ports = Run::root();
                let ended = {
                    let running = ports.port(HeadlessClaude::new(directory.clone(), agent.clone()));
                    let clock = ports.port(Stepped::starting(start, None));
                    let deadline = Deadline::after(clock.now(), timeout);
                    let prompt = Prompt::from(&filler.as_str().repeat(repeated));
                    let held = running.dispatch(&Job::due_by(deadline, Model::Haiku), &prompt)?;
                    let pid = held.pid();
                    let watched = Watch::over(&running, &clock).watched(held)?;
                    (watched, running.started(pid)?)
                };
                ports.finish().settled()?;
                Ok(ended)
            })
            .collect::<Result<Vec<_>, Failed>>()?;
        Ok(Debugged(stalled).to_string())
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}

#[test]
fn a_bell_lapses_until_its_agent_finishes_and_then_rings() -> Result<(), Failed> {
    let drifted = Cases::<Bells>::drifted(|case| {
        let directory = case.directory().to_path_buf();
        let Bells { sleeping, speaking, prompt, start, pause, quiet, timeout } = case.into_row();
        let stepped = Stepped::starting(start, None);
        let ports = Run::root();
        let woke = {
            let clock = ports.port(stepped.clone());
            let silent_agent = ports.port(HeadlessClaude::new(directory.clone(), sleeping));
            let speaking_agent = ports.port(HeadlessClaude::new(directory, speaking));
            let job = Job::due_by(Deadline::after(clock.now(), timeout), Model::Haiku);
            let silent = silent_agent.dispatch(&job, &Prompt::from(&prompt))?;
            let waited = clock.waited(pause, silent.bell());
            let began = Instant::now();
            let lapsed = (silent.bell().waited(quiet), began.elapsed() >= quiet);
            silent.end()?;
            let finished = speaking_agent.dispatch(&job, &Prompt::from(&prompt))?;
            let spoken = finished.bell().waited(quiet.saturating_mul(u32::from(u8::MAX)));
            finished.end()?;
            (waited, lapsed, spoken)
        };
        ports.finish().settled()?;
        Ok(Debugged((woke, stepped.waits())).to_string())
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}

#[test]
fn a_shell_call_returns_its_printing_and_whether_it_succeeded_failed_or_ran_past_its_deadline() -> Result<(), Failed> {
    let drifted = Cases::<Shells>::drifted(|case| {
        let directory = case.directory().to_path_buf();
        let Shells { program, calls } = case.into_row();
        let ports = Run::root();
        let ran = {
            let shell = ports.port(SystemShell);
            let clock = ports.port(SystemClock);
            calls
                .iter()
                .map(|called| {
                    let spoken = called.arguments.iter().map(|word| word.as_str().to_owned()).collect();
                    let deadline = Deadline::after(clock.now(), called.timeout);
                    shell.ran(&Call::new(program.as_str(), spoken, directory.clone(), deadline))
                })
                .collect::<Result<Vec<_>, _>>()?
        };
        let counted = ports.finish().settled()?.calls::<shell_ops::Ran>();
        Ok(Debugged((ran, counted)).to_string())
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}
