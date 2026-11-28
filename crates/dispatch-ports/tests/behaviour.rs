use std::path::{Path, PathBuf};

use dispatch_ports::{Call, DispatchFailure, Ran, Shell, SystemShell, shell_ops};
use port_core::{Calls, Run};
use serde::Deserialize;
use test_failure::{Cases, Failed, Golden};
use timer_ports::{Deadline, Timeout, Wall};

#[derive(Debug, Deserialize)]
struct Fixture {
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
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cases: Fixture = Cases::read(&manifest.join("tests/calls.toml"))?;
    let run = Run::root();
    let shown = {
        let shell = run.port(SystemShell);
        let before = Wall::now();
        let calls: Vec<Call> = cases.call.iter().map(|spoken| spoken.call(before)).collect();
        let fanned: Vec<Result<Ran, DispatchFailure>> = shell.fanned(&calls);
        let first = calls.first().ok_or("no calls")?;
        let alone = shell.ran(first)?;
        let ran: Vec<String> = fanned
            .into_iter()
            .zip(&cases.call)
            .map(|(ran, spoken)| {
                format!("{:?} {:?} -> {:?}\n", spoken.arguments, spoken.fed, ran.map_err(|failed| failed.to_string()))
            })
            .collect();
        let printed = alone.printed().map(ToString::to_string);
        format!("{}alone {printed:?} {}\n", ran.concat(), matches!(alone, Ran::Succeeded(_)))
    };
    let measured = run.finish();
    let snapshot = measured.snapshot();
    let counted = [snapshot.calls::<shell_ops::Fanned>(), snapshot.calls::<shell_ops::Ran>()];
    assert_eq!(counted, [Calls::new(1); 2]);
    measured.settled()?;
    Golden::at(manifest, "tests/ran.txt").checked(&shown)
}
