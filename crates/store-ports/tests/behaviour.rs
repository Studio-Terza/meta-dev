use std::num::NonZeroU32;
use std::path::PathBuf;

use design_messages::{CheckName, Fate, Halt, PullNumber, SessionCursors, SessionRecords, StreamSchema};
use dispatch_ports::{Pid, Tokens, Wall};
use file_ports::Offset;
use newtype::{Counter, Enumerated};
use port_core::Run;
use proptest::option;
use proptest::prelude::{any, prop_assert, prop_assert_eq, proptest};
use proptest::test_runner::{Config, RngSeed};
use sqlite_ports::SqliteOpener;
use store_ports::{Busy, Column, ColumnFailure, Counted, Opens, Store, Written};
use tempfile::TempDir;
use transcript_ports::{Authorship, Body, Cursor, Instant, SessionId, Uuid};

proptest! {
    #![proptest_config(Config { failure_persistence: None, rng_seed: RngSeed::Fixed(0), ..Config::default() })]
    #[test]
    fn a_reopened_store_resumes_each_session_where_it_was_left(
        session in any::<SessionId>(),
        cursor in any::<Cursor>(),
        store in "[a-z]{1,8}/[a-z]{1,8}",
    ) {
        let directory = TempDir::new()?;
        let path = directory.path().join(store);
        let run = Run::root();
        let first = run.port(SqliteOpener.file::<StreamSchema>(path.clone()));
        prop_assert!(path.metadata().is_err());
        let written: Written = first.write(Counted::<SessionRecords>::new(SessionRecords::read(&session, cursor, &[])))?;
        prop_assert_eq!(written, Written::zero().incremented());
        prop_assert!(path.metadata().is_ok());
        let reopened = run.port(SqliteOpener.file::<StreamSchema>(path));
        prop_assert_eq!(reopened.read(SessionCursors)?.resuming(&session), cursor);
    }

    #[test]
    fn a_uuid_column_reads_back(value in any::<Uuid>()) {
        prop_assert_eq!(Uuid::from_raw(value.raw()).into_iter().next(), Some(value));
    }

    #[test]
    fn a_session_column_reads_back(value in any::<SessionId>()) {
        prop_assert_eq!(SessionId::from_raw(value.raw()).into_iter().next(), Some(value));
    }

    #[test]
    fn an_instant_column_reads_back(value in any::<Instant>()) {
        prop_assert_eq!(Instant::from_raw(value.raw()).into_iter().next(), Some(value));
    }

    #[test]
    fn a_body_column_reads_back(value in any::<Body>()) {
        prop_assert_eq!(Body::from_raw(value.raw()).into_iter().next(), Some(value));
    }

    #[test]
    fn an_offset_column_reads_back(value in any::<Offset>()) {
        prop_assert_eq!(Offset::from_raw(value.raw()).into_iter().next(), Some(value));
    }

    #[test]
    fn an_optional_column_reads_back(value in option::of(any::<Body>())) {
        prop_assert_eq!(Option::<Body>::from_raw(value.raw()).into_iter().next(), Some(value));
    }

    #[test]
    fn a_pid_column_reads_back(value in any::<Pid>()) {
        prop_assert_eq!(Pid::from_raw(value.raw()).into_iter().next(), Some(value));
    }

    #[test]
    fn a_token_count_column_reads_back(value in any::<Tokens>()) {
        prop_assert_eq!(Tokens::from_raw(value.raw()).into_iter().next(), Some(value));
    }

    #[test]
    fn a_wall_time_column_reads_back(value in any::<Wall>()) {
        prop_assert_eq!(Wall::from_raw(value.raw()).into_iter().next(), Some(value));
    }

    #[test]
    fn a_pull_number_column_reads_back_and_refuses_what_is_not_one(number in any::<i64>()) {
        let fits = u32::try_from(number).into_iter().filter_map(NonZeroU32::new).map(PullNumber::new).next();
        prop_assert_eq!(PullNumber::from_raw(number).into_iter().next(), fits);
        prop_assert_eq!(fits.map(|pull| PullNumber::from_raw(pull.raw()).into_iter().next()), fits.map(Some));
    }

    #[test]
    fn a_check_name_column_reads_back_and_refuses_blank_and_control_text(text in "[ -~\n\u{7f}]{0,8}") {
        let named = CheckName::try_new(text.clone()).into_iter().next();
        prop_assert_eq!(&CheckName::from_raw(text).into_iter().next(), &named);
        prop_assert_eq!(named.clone().map(|name| CheckName::from_raw(name.raw()).into_iter().next()), named.map(Some));
    }

    #[test]
    fn a_count_beyond_a_pid_is_refused_as_unfit(number in i64::from(u32::MAX) + 1..i64::MAX) {
        prop_assert!(matches!(Pid::from_raw(number), Err(ColumnFailure::Unfit(_))));
    }

    #[test]
    fn an_unknown_spelling_is_refused_as_misspelled(text in "[a-z]{0,3}[A-Z ]{1,3}") {
        prop_assert!(matches!(Halt::from_raw(text.clone()), Err(ColumnFailure::Misspelled(_))));
        prop_assert!(matches!(Authorship::from_raw(text), Err(ColumnFailure::Misspelled(_))));
    }

    #[test]
    fn a_negative_offset_is_refused(number in i64::MIN..0) {
        prop_assert!(matches!(Offset::from_raw(number), Err(ColumnFailure::Refused(_))));
    }
}

#[test]
fn every_named_column_reads_back_as_its_variant() {
    let halts: Vec<Option<Halt>> =
        Halt::VARIANTS.iter().map(|halt| Halt::from_raw(halt.raw()).into_iter().next()).collect();
    let fates: Vec<Option<Fate>> =
        Fate::VARIANTS.iter().map(|fate| Fate::from_raw(fate.raw()).into_iter().next()).collect();
    let authors: Vec<Option<Authorship>> = [Authorship::Owner, Authorship::Lead]
        .iter()
        .map(|author| Authorship::from_raw(author.raw()).into_iter().next())
        .collect();
    assert_eq!(halts, Halt::VARIANTS.iter().copied().map(Some).collect::<Vec<Option<Halt>>>());
    assert_eq!(fates, Fate::VARIANTS.iter().copied().map(Some).collect::<Vec<Option<Fate>>>());
    assert_eq!(authors, [Authorship::Owner, Authorship::Lead].map(Some));
}

proptest! {
    #![proptest_config(Config { cases: 4, failure_persistence: None, ..Config::default() })]
    #[test]
    fn a_held_file_lock_refuses_every_other_handle_until_it_ends(store in "[a-z]{1,8}/[a-z]{1,8}") {
        let directory = TempDir::new()?;
        let path: PathBuf = directory.path().join(store);
        let run = Run::root();
        let (first, second) =
            (run.port(SqliteOpener.file::<StreamSchema>(path.clone())), run.port(SqliteOpener.file::<StreamSchema>(path)));
        let held = first.lock()?;
        prop_assert!(matches!(second.lock(), Err(Busy::Taken(_))));
        held.end()?;
        prop_assert!(second.lock().is_ok());
    }
}

#[test]
fn a_held_memory_lock_refuses_a_second_lock_until_it_is_dropped() -> Result<(), Busy> {
    let run = Run::root();
    let store = run.port(SqliteOpener.in_memory::<StreamSchema>());
    let held = store.lock()?;
    assert!(matches!(store.lock(), Err(Busy::Claimed)));
    drop(held);
    assert!(store.lock().is_ok());
    Ok(())
}
