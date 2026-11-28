use std::fs;
use std::path::Path;

use dispatch_ports::{Clock, Pause, SystemClock, clock_ops};
use port_core::{Calls, Run};
use serde::Deserialize;
use test_failure::Failed;

#[derive(Debug, Deserialize)]
struct Cases {
    pause: u32,
}

#[test]
fn the_system_clock_reads_the_time_and_pauses_forward() -> Result<(), Failed> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let cases: Cases = toml::from_str(&fs::read_to_string(directory.join("calls.toml"))?)?;
    let run = Run::root();
    let shown = {
        let clock = run.port(SystemClock);
        let before = clock.now();
        let paused = clock.paused(Pause::new(cases.pause));
        format!("paused forward {}\n", paused >= before)
    };
    let measured = run.finish();
    let snapshot = measured.snapshot();
    let counted = [snapshot.calls::<clock_ops::Now>(), snapshot.calls::<clock_ops::Paused>()];
    assert_eq!(counted, [Calls::new(1); 2]);
    measured.settled()?;
    let golden = directory.join("ran.txt");
    std::env::var_os("RECORD_GOLDENS").map_or(Ok(()), |_| fs::write(&golden, &shown))?;
    assert_eq!(shown, fs::read_to_string(golden)?);
    Ok(())
}
