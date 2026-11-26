use std::error::Error;
use std::time::Duration;

use dispatch_fakes::{Heeding, Stepped};
use dispatch_ports::{AgentRun, Clock, ClockFiller, Deadline, Pause, Pid, Timeout, Wall, Woke};
use port_core::Run;

#[derive(Debug, PartialEq, Eq)]
struct Rung {
    waited: Wall,
    left: Woke,
}

impl Rung {
    fn waited<F: ClockFiller>(run: &Run, clock: F) -> Rung {
        let agent = AgentRun::new(Pid::new(1), Deadline::after(Wall::new(0), Timeout::new(5)));
        agent.bell().ringer().ring();
        let waited = run.port(clock).waited(Pause::new(1), agent.bell());
        Rung { waited, left: agent.bell().waited(Duration::ZERO) }
    }
}

#[test]
fn a_heeding_clock_spends_a_rung_bell_while_a_stepped_clock_leaves_it_rung() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let rung = [
        Rung::waited(&run, Heeding::from(Stepped::starting(Wall::new(0), Pause::new(0)))),
        Rung::waited(&run, Stepped::starting(Wall::new(0), Pause::new(0))),
    ];
    run.finish().settled()?;
    let one = Wall::new(1);
    assert_eq!(rung, [Rung { waited: one, left: Woke::Lapsed }, Rung { waited: one, left: Woke::Rung }]);
    Ok(())
}
