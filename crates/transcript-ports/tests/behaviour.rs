#![deny(clippy::wildcard_enum_match_arm)]

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use file_ports::{Bytes, Disk, Memory};
use newtype::Counter;
use port_core::Run;
use proptest::prelude::{any, prop_assert, prop_assert_eq, proptest};
use proptest::sample::Index;
use serde::Deserialize;
use serde::de::IgnoredAny;
use test_failure::Failed;
use transcript_ports::transcripts_ops::{ReadFrom, Sessions};
use transcript_ports::{
    Authorship, Body, ClaudeCodeDir, Cursor, Instant, Kind, Line, Read, Record, SessionId, Transcripts, Uuid,
};

#[test]
fn each_transcript_line_reads_as_who_said_it_the_edge_it_leaves_or_the_kind_nobody_knows() -> Result<(), Failed> {
    let source = Path::new(file!());
    let table = source.with_file_name(source.file_stem().unwrap_or_default());
    let found = std::env::current_dir()?
        .ancestors()
        .map(|directory| directory.join(&table))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
    let bytes = fs::read(found)?;
    let mut rows = serde_json::Deserializer::from_slice(&bytes);
    let (fixtures, golden) = <(PathBuf, PathBuf)>::deserialize(&mut rows)?;
    let run = Run::root();
    let read = {
        let port = run.port(ClaudeCodeDir::new(Disk, std::env::current_dir()?.join(fixtures)));
        port.sessions()?
            .iter()
            .map(|session| {
                let lines: Vec<Line> = port.read_from(session, Cursor::default())?.into_iter().collect();
                let said: Vec<(Authorship, Uuid, SessionId, Option<Uuid>, Instant, Body)> = lines
                    .iter()
                    .filter_map(|line| match line {
                        Line::Said(record) => Some((
                            record.author(),
                            record.uuid().clone(),
                            record.session().clone(),
                            record.parent().cloned(),
                            record.stamp().clone(),
                            record.text().clone(),
                        )),
                        Line::Skipped(_) | Line::Unrecognised(_) => None,
                    })
                    .collect();
                let edges: Vec<(Option<Uuid>, Uuid)> = lines
                    .iter()
                    .filter_map(|line| match line {
                        Line::Skipped(skipped) => Some((skipped.parent().cloned(), skipped.uuid().clone())),
                        Line::Said(_) | Line::Unrecognised(_) => None,
                    })
                    .collect();
                let unknown: Vec<Kind> = lines
                    .iter()
                    .filter_map(|line| match line {
                        Line::Unrecognised(unrecognised) => Some(unrecognised.kind().clone()),
                        Line::Said(_) | Line::Skipped(_) => None,
                    })
                    .collect();
                Ok((session.stem().clone(), said, edges, unknown))
            })
            .collect::<Result<Vec<_>, Failed>>()?
    };
    run.finish().settled()?;
    assert_eq!(read, serde_json::from_slice::<Vec<_>>(&fs::read(golden)?)?);
    Ok(())
}

#[test]
fn each_text_reads_as_an_instant_a_kind_a_session_a_body_and_an_author_or_is_refused() -> Result<(), Failed> {
    let source = Path::new(file!());
    let table = source.with_file_name(source.file_stem().unwrap_or_default());
    let found = std::env::current_dir()?
        .ancestors()
        .map(|directory| directory.join(&table))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
    let bytes = fs::read(found)?;
    let mut rows = serde_json::Deserializer::from_slice(&bytes);
    IgnoredAny::deserialize(&mut rows)?;
    let (fixture, golden) = <(PathBuf, PathBuf)>::deserialize(&mut rows)?;
    let texts: Vec<String> = serde_json::from_slice(&fs::read(fixture)?)?;
    let read: Vec<_> = texts
        .iter()
        .map(|text| {
            (
                Instant::try_new(text.clone()).map_err(|refused| refused.to_string()),
                Kind::try_new(text.clone()).map_err(|refused| refused.to_string()),
                SessionId::try_new(text.clone()).map_err(|refused| refused.to_string()),
                Body::try_new(text.clone()).map_err(|refused| refused.to_string()),
                text.parse::<Authorship>().map_err(|refused| refused.to_string()),
            )
        })
        .collect();
    assert_eq!(read, serde_json::from_slice::<Vec<_>>(&fs::read(golden)?)?);
    Ok(())
}

proptest! {
    #[test]
    fn resuming_from_the_cursor_reads_every_record_exactly_once(cut in any::<Index>()) {
        let source = Path::new(file!());
        let table = source.with_file_name(source.file_stem().unwrap_or_default());
        let found = std::env::current_dir()?
            .ancestors()
            .map(|directory| directory.join(&table))
            .find(|candidate| candidate.is_file())
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
        let bytes = fs::read(found)?;
        let mut rows = serde_json::Deserializer::from_slice(&bytes);
        IgnoredAny::deserialize(&mut rows)?;
        IgnoredAny::deserialize(&mut rows)?;
        let (fixture, path) = <(PathBuf, PathBuf)>::deserialize(&mut rows)?;
        let directory = path.parent().map_or_default(Path::to_path_buf);
        let text = fs::read_to_string(fixture)?;
        let kept: String = text.chars().take(cut.index(text.chars().count())).collect();
        let run = Run::root();
        let (first, second, whole) = {
            let cut_port = run.port(ClaudeCodeDir::new(Memory::from_iter([(path.clone(), Bytes::from(&kept))]), directory.clone()));
            let whole_port = run.port(ClaudeCodeDir::new(Memory::from_iter([(path, Bytes::from(&text))]), directory));
            let sessions = whole_port.sessions()?;
            let session = sessions.first().ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
            let first: Read = cut_port.read_from(session, Cursor::default())?;
            let second = whole_port.read_from(session, first.cursor())?;
            let whole = whole_port.read_from(session, Cursor::default())?;
            (first, second, whole)
        };
        run.finish().settled()?;
        let records = |read: Read| -> Vec<Record> {
            read.into_iter().filter_map(|line| match line {
                Line::Said(record) => Some(record),
                Line::Skipped(_) | Line::Unrecognised(_) => None,
            }).collect()
        };
        let together: Vec<Record> = records(first).into_iter().chain(records(second)).collect();
        prop_assert_eq!(together, records(whole));
    }

    #[test]
    fn each_listing_and_each_read_is_counted_once(listings in 1..4_usize) {
        let source = Path::new(file!());
        let table = source.with_file_name(source.file_stem().unwrap_or_default());
        let found = std::env::current_dir()?
            .ancestors()
            .map(|directory| directory.join(&table))
            .find(|candidate| candidate.is_file())
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
        let bytes = fs::read(found)?;
        let mut rows = serde_json::Deserializer::from_slice(&bytes);
        let (fixtures, _) = <(PathBuf, IgnoredAny)>::deserialize(&mut rows)?;
        let run = Run::root();
        let sessions = {
            let port = run.port(ClaudeCodeDir::new(Disk, std::env::current_dir()?.join(fixtures)));
            let listed = std::iter::repeat_with(|| port.sessions()).take(listings).collect::<Result<Vec<_>, _>>()?;
            listed.iter().flatten().try_for_each(|session| port.read_from(session, Cursor::default()).map(drop))?;
            listed.into_iter().flatten().count()
        };
        let settled = run.finish().settled()?;
        prop_assert_eq!(settled.calls::<Sessions>().count(), listings);
        prop_assert_eq!(settled.calls::<ReadFrom>().count(), sessions);
    }

    #[test]
    fn a_body_holding_a_nul_is_refused_and_any_other_text_is_kept_whole(
        before in "[^\u{0}]{0,8}",
        after in "[^\u{0}]{0,8}",
        nul in "\u{0}",
    ) {
        prop_assert!(Body::try_new([before.clone(), nul, after.clone()].concat()).is_err());
        prop_assert_eq!(Body::try_new([before.clone(), after.clone()].concat())?.to_string(), [before, after].concat());
    }

    #[test]
    fn an_instant_is_rebuilt_from_its_display(instant in any::<Instant>()) {
        prop_assert_eq!(Instant::try_new(instant.to_string())?, instant);
    }

    #[test]
    fn a_kind_is_rebuilt_from_its_display(kind in any::<Kind>()) {
        prop_assert_eq!(Kind::try_new(kind.to_string())?, kind);
    }

    #[test]
    fn a_session_id_is_rebuilt_from_its_display(session in any::<SessionId>()) {
        prop_assert_eq!(SessionId::try_new(session.to_string())?, session);
    }
}
