use std::collections::BTreeMap;
use std::num::NonZeroU32;

use design_messages::{CheckName, Fate, Halt, PullNumber, SessionCursors, SessionRecords, StreamSchema};
use dispatch_ports::{Pid, Tokens, Wall};
use file_ports::Offset;
use fixture::{Failed, Properties};
use invariants::{Ignoring, ignored};
use newtype::{Counter, Enumerated};
use port_core::Run;
use proptest::collection::vec;
use proptest::option;
use proptest::prelude::{any, prop_assert_eq, proptest};
use proptest::sample::Index;
use proptest::strategy::{Strategy, ValueTree};
use proptest::test_runner::TestRunner;
use sqlite_ports::SqliteOpener;
use store_ports::{Busy, Column, Counted, Opens, Store, Written};
use tempfile::TempDir;
use transcript_ports::{Authorship, Body, Cursor, Instant, SessionId, Uuid};

ignored! {
    enum Ignored {
        Unread => "a raw value that does not read back is None, which the roundtrip compares",
    }
}

#[test]
fn a_store_opens_on_its_first_op_and_keeps_each_session_s_last_cursor_when_reopened() -> Result<(), Failed> {
    let settled = Properties::settled();
    let drawn = usize::try_from(settled.cases)?;
    let mut runner = TestRunner::new(settled);
    let sessions = vec(any::<SessionId>(), drawn).new_tree(&mut runner)?.current();
    let cursors = vec(any::<Cursor>(), drawn).new_tree(&mut runner)?.current();
    let revisits = vec(any::<Index>(), drawn).new_tree(&mut runner)?.current();
    let visits: Vec<(&SessionId, Cursor)> =
        revisits.iter().map(|at| at.get(&sessions)).chain(&sessions).zip(cursors.iter().cycle().copied()).collect();
    let scratch = TempDir::new()?;
    let named = scratch.path().file_name().unwrap_or_default();
    let place = scratch.path().join(named).join(named);
    let run = Run::root();
    let store = run.port(SqliteOpener.file::<StreamSchema>(place.clone()));
    let before = place.exists();
    let written = visits
        .iter()
        .map(|(session, cursor)| {
            store.write::<Counted<SessionRecords>>(Counted::new(SessionRecords::read(session, *cursor, &[])))
        })
        .collect::<Result<Vec<Written>, _>>()?;
    let last: BTreeMap<&SessionId, Cursor> = visits.iter().copied().collect();
    let changes = visits.iter().enumerate().filter(|(at, (session, cursor))| {
        visits
            .get(..*at)
            .and_then(|earlier| earlier.iter().rev().find(|(seen, _)| seen == session))
            .map(|(_, kept)| kept)
            != Some(cursor)
    });
    let reopened = run.port(SqliteOpener.file::<StreamSchema>(place.clone()));
    let read = reopened.read(SessionCursors)?;
    let unkept: Vec<_> = last.iter().filter(|(session, cursor)| read.resuming(session) != **cursor).collect();
    assert!(!before && place.exists());
    assert_eq!((Written::total(written.into_iter()).count(), unkept), (changes.count(), Vec::new()));
    Ok(())
}

proptest! {
    #![proptest_config(Properties::settled())]

    #[test]
    fn every_column_reads_back_what_it_wrote(uuid in any::<Uuid>(), session in any::<SessionId>(), instant in any::<Instant>(), body in any::<Body>(), offset in any::<Offset>(), optional in option::of(any::<Body>()), pid in any::<Pid>(), tokens in any::<Tokens>(), wall in any::<Wall>(), pull in any::<NonZeroU32>()) {
        prop_assert_eq!(Uuid::from_raw(uuid.raw()).ignoring(Ignored::Unread), Some(uuid));
        prop_assert_eq!(SessionId::from_raw(session.raw()).ignoring(Ignored::Unread), Some(session));
        prop_assert_eq!(Instant::from_raw(instant.raw()).ignoring(Ignored::Unread), Some(instant));
        prop_assert_eq!(Body::from_raw(body.raw()).ignoring(Ignored::Unread), Some(body));
        prop_assert_eq!(Offset::from_raw(offset.raw()).ignoring(Ignored::Unread), Some(offset));
        prop_assert_eq!(<Option<Body>>::from_raw(optional.raw()).ignoring(Ignored::Unread), Some(optional));
        prop_assert_eq!(Pid::from_raw(pid.raw()).ignoring(Ignored::Unread), Some(pid));
        prop_assert_eq!(Tokens::from_raw(tokens.raw()).ignoring(Ignored::Unread), Some(tokens));
        prop_assert_eq!(Wall::from_raw(wall.raw()).ignoring(Ignored::Unread), Some(wall));
        prop_assert_eq!(PullNumber::from_raw(PullNumber::new(pull).raw()).ignoring(Ignored::Unread), Some(PullNumber::new(pull)));
    }

    #[test]
    fn a_column_refuses_exactly_what_its_type_cannot_hold(number in any::<i64>(), text in any::<String>()) {
        prop_assert_eq!(Pid::from_raw(number).is_ok(), u32::try_from(number).is_ok());
        prop_assert_eq!(Offset::from_raw(number).is_ok(), u64::try_from(number).is_ok());
        prop_assert_eq!(PullNumber::from_raw(number).is_ok(), u32::try_from(number).is_ok_and(|pull| NonZeroU32::new(pull).is_some()));
        prop_assert_eq!(CheckName::from_raw(text.clone()).is_ok(), CheckName::try_new(text.clone()).is_ok());
        prop_assert_eq!(Authorship::from_raw(text.clone()).is_ok(), text.parse::<Authorship>().is_ok());
        prop_assert_eq!(Halt::from_raw(text.clone()).is_ok(), Halt::VARIANTS.iter().any(|halt| halt.raw() == text));
        prop_assert_eq!(Fate::from_raw(text.clone()).is_ok(), Fate::VARIANTS.iter().any(|fate| fate.raw() == text));
        let checked = CheckName::try_new(text).ignoring(Ignored::Unread);
        prop_assert_eq!(checked.clone().and_then(|name| CheckName::from_raw(name.raw()).ignoring(Ignored::Unread)), checked);
    }
}

#[test]
fn every_named_column_reads_back_as_its_variant() {
    let halts =
        Halt::VARIANTS.iter().filter(|halt| Halt::from_raw(halt.raw()).ignoring(Ignored::Unread) != Some(**halt));
    let fates =
        Fate::VARIANTS.iter().filter(|fate| Fate::from_raw(fate.raw()).ignoring(Ignored::Unread) != Some(**fate));
    let authors = [Authorship::Owner, Authorship::Lead];
    let unread =
        authors.iter().filter(|author| Authorship::from_raw(author.raw()).ignoring(Ignored::Unread) != Some(**author));
    assert_eq!((halts.count(), fates.count(), unread.count()), (usize::MIN, usize::MIN, usize::MIN));
}

#[test]
fn a_held_lock_refuses_other_file_handles_and_a_second_memory_lock_until_dropped() -> Result<(), Failed> {
    let scratch = TempDir::new()?;
    let place = scratch.path().join(scratch.path().file_name().unwrap_or_default());
    let run = Run::root();
    let (first, second, memory) = (
        run.port(SqliteOpener.file::<StreamSchema>(place.clone())),
        run.port(SqliteOpener.file::<StreamSchema>(place)),
        run.port(SqliteOpener.in_memory::<StreamSchema>()),
    );
    let (held, claimed) = (first.lock()?, memory.lock()?);
    assert!(matches!((second.lock(), memory.lock()), (Err(Busy::Taken(_)), Err(Busy::Claimed))));
    drop(held);
    drop(claimed);
    assert!(second.lock().is_ok() && memory.lock().is_ok());
    Ok(())
}
