use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::path::PathBuf;

use design_messages::{CheckName, EndedTasks, PullNumber, SessionCursors, SessionRecords, StreamSchema};
use file_ports::{Disk, Files, Offset};
use fixture::{Cases, Debugged, Failed, Fixture, Properties};
use invariants::{Ignoring, ignored};
use newtype::Counter;
use port_core::Run;
use proptest::collection::vec;
use proptest::prelude::{any, prop_assert_eq, proptest};
use proptest::sample::Index;
use proptest::strategy::{Strategy, ValueTree};
use proptest::test_runner::TestRunner;
use serde::Deserialize;
use sqlite_ports::SqliteOpener;
use store_ports::{Busy, Column, Counted, Opens, Parses, Store, Written};
use tempfile::TempDir;
use transcript_ports::{Authorship, Body, Cursor, Instant, SessionId, TaskId, Uuid};

ignored! {
    enum Ignored {
        Unread => "a raw value that does not read back is None, which the roundtrip compares",
    }
}

#[derive(Fixture, Deserialize, Debug)]
struct Schemas {
    older: PathBuf,
}

#[test]
fn a_store_kept_under_an_older_schema_is_rebuilt_under_the_current_one() -> Result<(), Failed> {
    let drifted = Cases::<Schemas>::drifted(|case| {
        let reading = Run::root();
        let text = reading.port(Disk).read_text(&case.directory().join(&case.row().older))?;
        reading.finish();
        let (older, current) = (SqliteOpener.parsed(text.as_ref())?, SqliteOpener.parsed(StreamSchema::SQL)?);
        let mut runner = TestRunner::new(Properties::settled());
        let (session, cursor) = (any::<SessionId>(), any::<Cursor>()).new_tree(&mut runner)?.current();
        let scratch = TempDir::new()?;
        let place = scratch.path().join(scratch.path().file_name().unwrap_or_default());
        let run = Run::root();
        let kept = run.port(SqliteOpener.file(&older, place.clone()));
        let written =
            kept.write::<Counted<SessionRecords>>(Counted::new(SessionRecords::read(&session, cursor, &[])))?;
        let before = (kept.read(SessionCursors)?.resuming(&session), kept.read(EndedTasks));
        drop(kept);
        let rebuilt = run.port(SqliteOpener.file(&current, place));
        let after = (rebuilt.read(SessionCursors)?.resuming(&session), rebuilt.read(EndedTasks));
        drop(rebuilt);
        run.finish();
        Ok(Debugged((older, current, cursor, written, before, after)).to_string())
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
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
    let schema = SqliteOpener.parsed(StreamSchema::SQL)?;
    let run = Run::root();
    let store = run.port(SqliteOpener.file(&schema, place.clone()));
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
    let reopened = run.port(SqliteOpener.file(&schema, place.clone()));
    let read = reopened.read(SessionCursors)?;
    let unkept: Vec<_> = last.iter().filter(|(session, cursor)| read.resuming(session) != **cursor).collect();
    assert!(!before && place.exists());
    assert_eq!((Written::total(written.into_iter()).count(), unkept), (changes.count(), Vec::new()));
    Ok(())
}

proptest! {
    #![proptest_config(Properties::settled())]

    #[test]
    fn every_column_reads_back_what_it_wrote(uuid in any::<Uuid>(), session in any::<SessionId>(), instant in any::<Instant>(), body in any::<Body>(), offset in any::<Offset>(), task in any::<TaskId>(), pull in any::<NonZeroU32>()) {
        prop_assert_eq!(Uuid::from_raw(uuid.raw()).ignoring(Ignored::Unread), Some(uuid));
        prop_assert_eq!(SessionId::from_raw(session.raw()).ignoring(Ignored::Unread), Some(session));
        prop_assert_eq!(Instant::from_raw(instant.raw()).ignoring(Ignored::Unread), Some(instant));
        prop_assert_eq!(Body::from_raw(body.raw()).ignoring(Ignored::Unread), Some(body));
        prop_assert_eq!(Offset::from_raw(offset.raw()).ignoring(Ignored::Unread), Some(offset));
        prop_assert_eq!(TaskId::from_raw(task.raw()).ignoring(Ignored::Unread), Some(task));
        prop_assert_eq!(PullNumber::from_raw(PullNumber::new(pull).raw()).ignoring(Ignored::Unread), Some(PullNumber::new(pull)));
    }

    #[test]
    fn a_column_refuses_exactly_what_its_type_cannot_hold(number in any::<i64>(), text in any::<String>()) {
        prop_assert_eq!(Offset::from_raw(number).is_ok(), u64::try_from(number).is_ok());
        prop_assert_eq!(PullNumber::from_raw(number).is_ok(), u32::try_from(number).is_ok_and(|pull| NonZeroU32::new(pull).is_some()));
        prop_assert_eq!(CheckName::from_raw(text.clone()).is_ok(), CheckName::try_new(text.clone()).is_ok());
        prop_assert_eq!(Authorship::from_raw(text.clone()).is_ok(), text.parse::<Authorship>().is_ok());
        prop_assert_eq!(TaskId::from_raw(text.clone()).is_ok(), TaskId::try_new(text.clone()).is_ok());
        let checked = CheckName::try_new(text).ignoring(Ignored::Unread);
        prop_assert_eq!(checked.clone().and_then(|name| CheckName::from_raw(name.raw()).ignoring(Ignored::Unread)), checked);
    }
}

#[test]
fn every_named_column_reads_back_as_its_variant() {
    let authors = [Authorship::Owner, Authorship::Lead];
    let unread =
        authors.iter().filter(|author| Authorship::from_raw(author.raw()).ignoring(Ignored::Unread) != Some(**author));
    assert_eq!(unread.count(), usize::MIN);
}

#[test]
fn a_held_lock_refuses_other_file_handles_and_a_second_memory_lock_until_dropped() -> Result<(), Failed> {
    let scratch = TempDir::new()?;
    let place = scratch.path().join(scratch.path().file_name().unwrap_or_default());
    let schema = SqliteOpener.parsed(StreamSchema::SQL)?;
    let run = Run::root();
    let (first, second, memory) = (
        run.port(SqliteOpener.file(&schema, place.clone())),
        run.port(SqliteOpener.file(&schema, place)),
        run.port(SqliteOpener.in_memory(&schema)),
    );
    let (held, claimed) = (first.lock()?, memory.lock()?);
    assert!(matches!((second.lock(), memory.lock()), (Err(Busy::Taken(_)), Err(Busy::Claimed))));
    drop(held);
    drop(claimed);
    assert!(second.lock().is_ok() && memory.lock().is_ok());
    Ok(())
}
