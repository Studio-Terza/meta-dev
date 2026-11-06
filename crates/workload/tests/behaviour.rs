use std::convert::Infallible;
use std::sync::{Mutex, MutexGuard, PoisonError};

use newtype::Counter;
use port_core::{Calls, Pass, Run, port};
use workload::{Bound, Complexity, Failure, Harness, Scale, Workload};

#[port]
trait Ledger {
    fn tick(&self);

    fn tock(&self);
}

#[derive(Debug, Default)]
struct Books;

impl LedgerFiller for Books {
    fn tick(&self, _pass: Pass<'_, ledger_ops::Tick>) {}

    fn tock(&self, _pass: Pass<'_, ledger_ops::Tock>) {}
}

static SOLE: Mutex<()> = Mutex::new(());

struct Serial;

impl Serial {
    fn held() -> MutexGuard<'static, ()> {
        SOLE.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

struct Steady;

struct Greedy;

struct Quadratic;

struct Broken;

impl Steady {
    fn ticks(scale: Scale) -> usize {
        scale.count().saturating_mul(2)
    }

    fn ticked(run: &Run, ticks: usize) -> usize {
        let port = run.port(Books);
        (0..ticks).for_each(|_| Ledger::tick(&port));
        Ledger::tock(&port);
        ticks
    }
}

impl Workload for Steady {
    type Fixture = usize;
    type Done = usize;
    type Failure = Infallible;
    type Scaled = ledger_ops::Tick;
    type Listed = ledger_ops::Tock;

    const CLASS: Complexity = Complexity::Linear;

    fn fixture(scale: Scale) -> usize {
        Steady::ticks(scale)
    }

    fn inputs(scale: Scale) -> Calls {
        Calls::new(Steady::ticks(scale))
    }

    fn run(run: &Run, ticks: usize) -> Result<usize, Infallible> {
        Ok(Steady::ticked(run, ticks))
    }
}

impl Workload for Greedy {
    type Fixture = usize;
    type Done = usize;
    type Failure = Infallible;
    type Scaled = ledger_ops::Tick;
    type Listed = ledger_ops::Tock;

    const CLASS: Complexity = Complexity::Linear;

    fn fixture(scale: Scale) -> usize {
        Steady::ticks(scale).saturating_add(1)
    }

    fn inputs(scale: Scale) -> Calls {
        Calls::new(Steady::ticks(scale))
    }

    fn run(run: &Run, ticks: usize) -> Result<usize, Infallible> {
        Ok(Steady::ticked(run, ticks))
    }
}

impl Workload for Quadratic {
    type Fixture = usize;
    type Done = usize;
    type Failure = Infallible;
    type Scaled = ledger_ops::Tick;
    type Listed = ledger_ops::Tock;

    const CLASS: Complexity = Complexity::Linear;

    fn fixture(scale: Scale) -> usize {
        scale.count().saturating_mul(scale.count())
    }

    fn inputs(_scale: Scale) -> Calls {
        Calls::new(usize::MAX)
    }

    fn run(run: &Run, ticks: usize) -> Result<usize, Infallible> {
        Ok(Steady::ticked(run, ticks))
    }

    fn bounds(_scale: Scale) -> Vec<Bound> {
        Vec::new()
    }
}

impl Workload for Broken {
    type Fixture = ();
    type Done = usize;
    type Failure = Calls;
    type Scaled = ledger_ops::Tick;
    type Listed = ledger_ops::Tock;

    const CLASS: Complexity = Complexity::Linear;

    fn fixture(_scale: Scale) {}

    fn inputs(_scale: Scale) -> Calls {
        Calls::zero()
    }

    fn run(_run: &Run, _fixture: ()) -> Result<usize, Calls> {
        Err(Calls::zero())
    }
}

#[test]
fn a_workload_within_its_bounds_and_class_passes() {
    let _serial = Serial::held();
    assert!(Harness::<Steady>::verified().is_ok());
}

#[test]
fn a_workload_calling_an_op_past_its_bound_fails() {
    let _serial = Serial::held();
    assert!(matches!(Harness::<Greedy>::verified(), Err(Failure::Exceeded { .. })));
}

#[test]
fn a_linear_workload_whose_calls_grow_faster_than_its_scale_fails() {
    let _serial = Serial::held();
    assert!(matches!(Harness::<Quadratic>::verified(), Err(Failure::Superlinear { .. })));
}

#[test]
fn a_workload_whose_run_fails_fails_the_harness() {
    let _serial = Serial::held();
    assert!(matches!(Harness::<Broken>::verified(), Err(Failure::Ran(_))));
}
