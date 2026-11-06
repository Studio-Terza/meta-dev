use laws::law;
use port_core::{Calls, Entry, Op, OpName, Run, Snapshot};

#[derive(Debug)]
struct ReadSession;

impl Op for ReadSession {
    type Port = Lines;
    const NAME: OpName = OpName::new("read_session");
}

#[derive(Debug)]
struct WriteSession;

impl Op for WriteSession {
    type Port = Lines;
    const NAME: OpName = OpName::new("write_session");
}

trait Sessions {
    fn read_session(&self, entry: Entry<ReadSession>) -> usize;
    fn write_session(&self, entry: Entry<WriteSession>) -> usize;
}

#[derive(Debug)]
struct Lines(usize);

impl Sessions for Lines {
    fn read_session(&self, _entry: Entry<ReadSession>) -> usize {
        self.0
    }

    fn write_session(&self, _entry: Entry<WriteSession>) -> usize {
        self.0
    }
}

law!(every_call_is_counted_once_for_its_own_op, (0usize..16, 0usize..16), counts, {
    let (reads, writes) = counts;
    let port = Run::default().meter(Lines(1));
    (0..reads).for_each(|_| {
        port.call(Sessions::read_session);
    });
    (0..writes).for_each(|_| {
        port.call(Sessions::write_session);
    });
    let tally = port.tally().snapshot();
    laws::proptest::prop_assert_eq!(
        (tally.calls::<ReadSession>(), tally.calls::<WriteSession>()),
        (Calls::new(reads), Calls::new(writes))
    );
});

#[test]
fn a_call_returns_what_the_body_returns() {
    let port = Run::default().meter(Lines(3));
    assert_eq!(port.call(Sessions::read_session), 3);
}

#[test]
fn the_snapshot_lists_only_the_ops_that_were_called() {
    let port = Run::default().meter(Lines(1));
    port.call(Sessions::write_session);
    let names: Vec<OpName> = port.tally().snapshot().iter().map(|counted| counted.name()).collect();
    assert_eq!(names, vec![WriteSession::NAME]);
}

#[test]
fn meters_of_one_run_share_a_tally_and_separate_runs_do_not() {
    let run = Run::default();
    let first = run.meter(Lines(1));
    let second = run.meter(Lines(2));
    let other_run = Run::default();
    let other = other_run.meter(Lines(3));
    first.call(Sessions::read_session);
    second.call(Sessions::read_session);
    assert_eq!(first.tally().snapshot().calls::<ReadSession>(), Calls::new(2));
    assert_eq!(second.tally().snapshot().calls::<ReadSession>(), Calls::new(2));
    assert_eq!(run.snapshot().calls::<ReadSession>(), Calls::new(2));
    assert_eq!(other.tally().snapshot().calls::<ReadSession>(), Calls::new(0));
    assert_eq!(other_run.snapshot().calls::<ReadSession>(), Calls::new(0));
}

#[test]
fn calls_from_spawned_threads_are_counted() {
    let run = Run::default();
    let port = run.meter(Lines(1));
    std::thread::scope(|scope| {
        (0..4).for_each(|_| {
            scope.spawn(|| port.call(Sessions::read_session));
        });
    });
    assert_eq!(run.snapshot().calls::<ReadSession>(), Calls::new(4));
}

#[test]
fn a_snapshot_does_not_change_after_it_is_taken() {
    let run = Run::default();
    let port = run.meter(Lines(1));
    port.call(Sessions::read_session);
    let before: Snapshot = run.snapshot();
    port.call(Sessions::read_session);
    port.call(Sessions::write_session);
    assert_eq!(before.calls::<ReadSession>(), Calls::new(1));
    assert_eq!(before.calls::<WriteSession>(), Calls::new(0));
    assert_eq!(before.iter().count(), 1);
    assert_eq!(run.snapshot().calls::<ReadSession>(), Calls::new(2));
}

#[derive(Debug)]
struct ListSessions;

impl Op for ListSessions {
    type Port = Lines;
    const NAME: OpName = ReadSession::NAME;
}

#[test]
fn ops_with_one_name_count_separately() {
    let run = Run::default();
    let port = run.meter(Lines(1));
    port.call(Sessions::read_session);
    port.call(|_, _entry: Entry<ListSessions>| ());
    port.call(|_, _entry: Entry<ListSessions>| ());
    let snapshot = run.snapshot();
    assert_eq!(snapshot.calls::<ReadSession>(), Calls::new(1));
    assert_eq!(snapshot.calls::<ListSessions>(), Calls::new(2));
    assert_eq!(snapshot.iter().count(), 2);
}

#[derive(Debug)]
struct Alpha;

impl Op for Alpha {
    type Port = Lines;
    const NAME: OpName = OpName::new("alpha");
}

#[test]
fn the_snapshot_lists_ops_by_name_then_type_name_whatever_the_call_order() {
    let run = Run::default();
    let port = run.meter(Lines(1));
    port.call(Sessions::write_session);
    port.call(|_, _entry: Entry<ListSessions>| ());
    port.call(Sessions::read_session);
    port.call(|_, _entry: Entry<Alpha>| ());
    let names: Vec<OpName> = run.snapshot().iter().map(|counted| counted.name()).collect();
    assert_eq!(names, vec![Alpha::NAME, ReadSession::NAME, ReadSession::NAME, WriteSession::NAME]);
}
