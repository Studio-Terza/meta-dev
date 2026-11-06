use port_core::{Calls, Entry, Op, OpName, Run};

#[derive(Debug)]
struct Lines;

#[derive(Debug)]
struct Read;

impl Op for Read {
    type Port = Lines;
    const NAME: OpName = OpName::new("read");
}

#[test]
fn a_call_on_another_thread_is_counted() {
    let run = Run::default();
    let port = run.meter(Lines);
    std::thread::scope(|scope| {
        scope.spawn(|| port.call(|_, _entry: Entry<Read>| ()));
    });
    assert_eq!(port.tally().snapshot().calls::<Read>(), Calls::new(1));
}

#[test]
fn a_meter_counts_only_its_own_calls() {
    let first = Run::default().meter(Lines);
    let second = Run::default().meter(Lines);
    first.call(|_, _entry: Entry<Read>| ());
    second.call(|_, _entry: Entry<Read>| ());
    assert_eq!(first.tally().snapshot().calls::<Read>(), Calls::new(1));
}

#[test]
fn a_tally_taken_after_a_run_does_not_change_with_later_runs() {
    let run = || {
        let run = Run::default();
        run.meter(Lines).call(|_, _entry: Entry<Read>| ());
        run.snapshot()
    };
    let scale_one = run();
    let _scale_two = run();
    assert_eq!(scale_one.calls::<Read>(), Calls::new(1));
}

#[derive(Debug)]
struct ReadAgain;

impl Op for ReadAgain {
    type Port = Lines;
    const NAME: OpName = OpName::new("read");
}

#[test]
fn two_ops_of_one_port_never_share_a_count() {
    let port = Run::default().meter(Lines);
    port.call(|_, _entry: Entry<ReadAgain>| ());
    assert_eq!(port.tally().snapshot().calls::<Read>(), Calls::new(0));
}
