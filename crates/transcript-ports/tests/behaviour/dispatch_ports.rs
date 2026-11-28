use std::path::PathBuf;

use dispatch_ports::{Call, Clock, Deadline, Pause, Shell, SystemClock, SystemShell, Timeout, clock_ops, shell_ops};
use fixture::{Cases, Debugged, Failed, Fixture, Spelled};
use port_core::Run;
use serde::Deserialize;

#[derive(Fixture, Deserialize, Debug)]
struct Shells {
    pause: Pause,
    calls: Vec<Spoken>,
}

#[derive(Deserialize, Debug)]
struct Spoken {
    program: Spelled,
    arguments: Vec<Spelled>,
    #[serde(default)]
    fed: Option<Spelled>,
    timeout: Timeout,
}

#[test]
fn each_shell_call_returns_its_printing_and_whether_it_succeeded_failed_or_ran_past_its_deadline() -> Result<(), Failed>
{
    let drifted = Cases::<Shells>::drifted(|case| {
        let directory = case.directory().to_path_buf();
        let Shells { pause, calls } = case.into_row();
        let run = Run::root();
        let observed = {
            let shell = run.port(SystemShell);
            let clock = run.port(SystemClock);
            let before = clock.now();
            let built: Vec<Call> = calls
                .iter()
                .map(|spoken| {
                    let program: &'static str = Box::leak(spoken.program.as_str().to_owned().into_boxed_str());
                    let arguments = spoken.arguments.iter().map(ToString::to_string).collect();
                    let call = Call::new(
                        program,
                        arguments,
                        PathBuf::from(&directory),
                        Deadline::after(before, spoken.timeout),
                    );
                    spoken.fed.as_ref().map_or_else(|| call.clone(), |fed| call.clone().fed(fed.to_string()))
                })
                .collect();
            let fanned: Vec<_> =
                shell.fanned(&built).into_iter().map(|ran| ran.map_err(|failed| failed.to_string())).collect();
            let alone: Vec<_> =
                built.first().map(|first| shell.ran(first).map_err(|failed| failed.to_string())).into_iter().collect();
            let forward = clock.paused(pause) >= before;
            (fanned, alone, forward)
        };
        let snapshot = run.finish();
        let counted = [
            snapshot.calls::<shell_ops::Fanned>(),
            snapshot.calls::<shell_ops::Ran>(),
            snapshot.calls::<clock_ops::Now>(),
            snapshot.calls::<clock_ops::Paused>(),
        ];
        Ok(Debugged((observed, counted)).to_string())
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}
