#![deny(clippy::wildcard_enum_match_arm)]

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use file_ports::{Bytes, Disk, Memory};
use newtype::Counter;
use port_core::Run;
use proptest::prelude::{any, prop_assert, prop_assert_eq, proptest};
use proptest::sample::Index;
use serde_json::Value;
use test_failure::Failed;
use transcript_ports::transcripts_ops::{ReadFrom, Sessions};
use transcript_ports::{
    Authorship, Body, ClaudeCodeDir, Cursor, Instant, Kind, Line, Read, Record, SessionId, Transcripts,
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
    let rows: Vec<Vec<String>> = serde_json::from_slice(&fs::read(found)?)?;
    let [transcripts, ..] = rows.as_slice() else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let [fixtures, golden, tab, newline] = transcripts.as_slice() else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let run = Run::root();
    let shown = {
        let port = run.port(ClaudeCodeDir::new(Disk, std::env::current_dir()?.join(fixtures)));
        port.sessions()?
            .iter()
            .map(|session| {
                let read = port.read_from(session, Cursor::default())?;
                let lines = read.into_iter().map(|line| match line {
                    Line::Said(record) => [
                        record.author().to_string(),
                        record.uuid().to_string(),
                        record.session().to_string(),
                        record.parent().map_or_default(ToString::to_string),
                        record.stamp().to_string(),
                        record.text().to_string(),
                    ]
                    .join(tab),
                    Line::Skipped(skipped) => {
                        [skipped.parent().map_or_default(ToString::to_string), skipped.uuid().to_string()].join(tab)
                    }
                    Line::Unrecognised(unrecognised) => unrecognised.kind().to_string(),
                });
                Ok([session.stem().to_string()].into_iter().chain(lines).collect::<Vec<String>>().join(newline))
            })
            .collect::<Result<Vec<String>, Failed>>()?
    };
    run.finish().settled()?;
    assert_eq!([shown.join(newline), newline.clone()].concat(), fs::read_to_string(golden)?);
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
    let rows: Vec<Vec<String>> = serde_json::from_slice(&fs::read(found)?)?;
    let [_, checked, ..] = rows.as_slice() else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let [fixture, golden, tab, newline] = checked.as_slice() else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let texts: Vec<String> = serde_json::from_slice(&fs::read(fixture)?)?;
    let read: Vec<String> = texts
        .iter()
        .map(|text| {
            let instant = match Instant::try_new(text.clone()) {
                Ok(read) => read.to_string(),
                Err(refused) => refused.to_string(),
            };
            let kind = match Kind::try_new(text.clone()) {
                Ok(read) => read.to_string(),
                Err(refused) => refused.to_string(),
            };
            let session = match SessionId::try_new(text.clone()) {
                Ok(read) => read.to_string(),
                Err(refused) => refused.to_string(),
            };
            let body = match Body::try_new(text.clone()) {
                Ok(read) => read.to_string(),
                Err(refused) => refused.to_string(),
            };
            let author = match text.parse::<Authorship>() {
                Ok(read) => read.to_string(),
                Err(refused) => refused.to_string(),
            };
            [instant, kind, session, body, author].join(tab)
        })
        .collect();
    assert_eq!([read.join(newline), newline.clone()].concat(), fs::read_to_string(golden)?);
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
        let rows: Vec<Vec<String>> = serde_json::from_slice(&fs::read(found)?)?;
        let [_, _, resumed, ..] = rows.as_slice() else {
            return Err(io::Error::from(io::ErrorKind::InvalidData).into());
        };
        let [fixture, directory, file] = resumed.as_slice() else {
            return Err(io::Error::from(io::ErrorKind::InvalidData).into());
        };
        let text = fs::read_to_string(fixture)?;
        let kept: String = text.chars().take(cut.index(text.chars().count())).collect();
        let path = PathBuf::from(directory).join(file);
        let run = Run::root();
        let (first, second, whole) = {
            let cut_port = run.port(ClaudeCodeDir::new(Memory::from_iter([(path.clone(), Bytes::from(&kept))]), PathBuf::from(directory)));
            let whole_port = run.port(ClaudeCodeDir::new(Memory::from_iter([(path, Bytes::from(&text))]), PathBuf::from(directory)));
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
        let rows: Vec<Vec<String>> = serde_json::from_slice(&fs::read(found)?)?;
        let [transcripts, ..] = rows.as_slice() else {
            return Err(io::Error::from(io::ErrorKind::InvalidData).into());
        };
        let [fixtures, ..] = transcripts.as_slice() else {
            return Err(io::Error::from(io::ErrorKind::InvalidData).into());
        };
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
    fn a_record_of_a_kind_nobody_knows_is_unrecognised_and_never_said(kind in "[a-z]{1,8}-unheard") {
        let source = Path::new(file!());
        let table = source.with_file_name(source.file_stem().unwrap_or_default());
        let found = std::env::current_dir()?
            .ancestors()
            .map(|directory| directory.join(&table))
            .find(|candidate| candidate.is_file())
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
        let rows: Vec<Vec<String>> = serde_json::from_slice(&fs::read(found)?)?;
        let [_, _, _, unheard, ..] = rows.as_slice() else {
            return Err(io::Error::from(io::ErrorKind::InvalidData).into());
        };
        let [template, key, newline, directory, file] = unheard.as_slice() else {
            return Err(io::Error::from(io::ErrorKind::InvalidData).into());
        };
        let mut line: Value = serde_json::from_str(template)?;
        if let Value::Object(fields) = &mut line {
            fields.insert(key.clone(), Value::from(kind.clone()));
        }
        let held = Memory::from_iter([(PathBuf::from(directory).join(file), Bytes::from(&[line.to_string(), newline.clone()].concat()))]);
        let run = Run::root();
        let read = {
            let port = run.port(ClaudeCodeDir::new(held, PathBuf::from(directory)));
            let sessions = port.sessions()?;
            let session = sessions.first().ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
            port.read_from(session, Cursor::default())?
        };
        run.finish().settled()?;
        let kinds: Vec<Option<String>> = read.into_iter().map(|each| match each {
            Line::Unrecognised(unrecognised) => Some(unrecognised.kind().to_string()),
            Line::Said(_) | Line::Skipped(_) => None,
        }).collect();
        prop_assert_eq!(kinds, [Some(kind)]);
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
