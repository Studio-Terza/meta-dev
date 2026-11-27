use std::fs;
use std::path::{Path, PathBuf};

use dispatch_ports::{
    Call, Clock, Deadline, DispatchFailure, Pause, Ran, Shell, SystemClock, SystemShell, Timeout, Wall, clock_ops,
    shell_ops,
};
use port_core::{Calls, Run};
use serde::Deserialize;
use test_failure::Failed;

#[derive(Debug, Deserialize)]
struct Cases {
    pause: u32,
    call: Vec<Spoken>,
}

#[derive(Debug, Deserialize)]
struct Spoken {
    program: String,
    arguments: Vec<String>,
    fed: Option<String>,
    seconds: u32,
}

impl Spoken {
    fn call(&self, now: Wall) -> Call {
        let program: &'static str = self.program.clone().leak();
        let deadline = Deadline::after(now, Timeout::new(self.seconds));
        let call = Call::new(program, self.arguments.clone(), PathBuf::from(env!("CARGO_MANIFEST_DIR")), deadline);
        match self.fed.clone() {
            Some(fed) => call.fed(fed),
            None => call,
        }
    }
}

#[test]
fn each_shell_call_returns_its_printing_and_whether_it_succeeded_failed_or_ran_past_its_deadline() -> Result<(), Failed>
{
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let cases: Cases = toml::from_str(&fs::read_to_string(directory.join("calls.toml"))?)?;
    let run = Run::root();
    let shown = {
        let shell = run.port(SystemShell);
        let clock = run.port(SystemClock);
        let before = clock.now();
        let calls: Vec<Call> = cases.call.iter().map(|spoken| spoken.call(before)).collect();
        let fanned: Vec<Result<Ran, DispatchFailure>> = shell.fanned(&calls);
        let first = calls.first().ok_or("no calls")?;
        let alone = shell.ran(first)?;
        let paused = clock.paused(Pause::new(cases.pause));
        let ran: Vec<String> = fanned
            .into_iter()
            .zip(&cases.call)
            .map(|(ran, spoken)| {
                format!("{:?} {:?} -> {:?}\n", spoken.arguments, spoken.fed, ran.map_err(|failed| failed.to_string()))
            })
            .collect();
        let printed = alone.printed().map(ToString::to_string);
        format!(
            "{}alone {printed:?} {}\npaused forward {}\n",
            ran.concat(),
            matches!(alone, Ran::Succeeded(_)),
            paused >= before
        )
    };
    let measured = run.finish();
    let snapshot = measured.snapshot();
    let counted = [
        snapshot.calls::<shell_ops::Fanned>(),
        snapshot.calls::<shell_ops::Ran>(),
        snapshot.calls::<clock_ops::Now>(),
        snapshot.calls::<clock_ops::Paused>(),
    ];
    assert_eq!(counted, [Calls::new(1); 4]);
    measured.settled()?;
    let golden = directory.join("ran.txt");
    std::env::var_os("RECORD_GOLDENS").map_or(Ok(()), |_| fs::write(&golden, &shown))?;
    assert_eq!(shown, fs::read_to_string(golden)?);
    Ok(())
}
