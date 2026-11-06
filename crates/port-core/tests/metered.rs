use laws::law;
use port_core::{Calls, Entry, Metered, Op, OpName};

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
    let port = Metered::new(Lines(1));
    port.tally().reset();
    (0..reads).for_each(|_| {
        port.call(Sessions::read_session);
    });
    (0..writes).for_each(|_| {
        port.call(Sessions::write_session);
    });
    let tally = port.tally();
    laws::proptest::prop_assert_eq!(
        (tally.calls(ReadSession::NAME), tally.calls(WriteSession::NAME)),
        (Calls::new(reads), Calls::new(writes))
    );
});

#[test]
fn a_call_returns_what_the_body_returns() {
    let port = Metered::new(Lines(3));
    assert_eq!(port.call(Sessions::read_session), 3);
}

#[test]
fn the_snapshot_lists_only_the_ops_that_were_called() {
    let port = Metered::new(Lines(1));
    port.call(Sessions::write_session);
    let names: Vec<OpName> = port.tally().snapshot().keys().copied().collect();
    assert_eq!(names, vec![WriteSession::NAME]);
}

#[test]
fn two_meters_of_one_port_count_into_one_tally() {
    let first = Metered::new(Lines(1));
    let second = Metered::new(Lines(2));
    first.tally().reset();
    first.call(Sessions::read_session);
    second.call(Sessions::read_session);
    assert_eq!(first.tally().calls(ReadSession::NAME), Calls::new(2));
    assert_eq!(second.tally().calls(ReadSession::NAME), Calls::new(2));
}

#[test]
fn a_reset_clears_the_tally() {
    let port = Metered::new(Lines(1));
    port.call(Sessions::read_session);
    port.tally().reset();
    assert_eq!(port.tally().snapshot(), std::collections::BTreeMap::new());
}
