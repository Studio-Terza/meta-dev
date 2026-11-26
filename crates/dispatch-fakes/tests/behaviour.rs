use std::error::Error;
use std::time::Duration;

use dispatch_fakes::Stepped;
use dispatch_ports::{AgentRun, Clock, Deadline, Pause, Pid, Timeout, Wall, Woke};
use port_core::Run;

#[test]
fn a_heeding_clock_spends_a_rung_bell_while_a_starting_clock_leaves_it_rung() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let left = [Stepped::heeding(Wall::new(0), Pause::new(0)), Stepped::starting(Wall::new(0), Pause::new(0))].map(
        |stepped| {
            let agent = AgentRun::new(Pid::new(1), Deadline::after(Wall::new(0), Timeout::new(5)));
            agent.bell().ringer().ring();
            let waited = run.port(stepped).waited(Pause::new(1), agent.bell());
            (waited, agent.bell().waited(Duration::ZERO))
        },
    );
    run.finish().settled()?;
    assert_eq!(left, [(Wall::new(1), Woke::Lapsed), (Wall::new(1), Woke::Rung)]);
    Ok(())
}
