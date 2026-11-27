#![deny(clippy::wildcard_enum_match_arm)]

use std::collections::BTreeSet;

use doubles::Memory;
use file_ports::{Bytes, Disk, Files};
use fixture::{Cases, Debugged, Failed, Fixture, Properties, Spelled};
use invariants::{Ignoring, ignored};
use port_core::Run;
use proptest::prelude::{any, prop_assert_eq, proptest};
use serde::Deserialize;
use transcript_ports::transcripts_ops::{ReadFrom, Sessions};
use transcript_ports::{
    Authorship, Body, ClaudeCodeDir, Cursor, Instant, Kind, Line, Read, Session, SessionId, Transcripts,
};

ignored! {
    enum Ignored {
        Refused => "a text the check or parse refuses is the None the golden shows",
    }
}

#[derive(Fixture, Deserialize, Debug)]
struct Transcript {
    check: Check,
    #[serde(default)]
    texts: Vec<Spelled>,
}

#[derive(Deserialize, Clone, Copy, Debug)]
enum Check {
    Classified,
    Resumed,
    Admitted,
}

#[test]
fn every_transcript_case_reads_as_its_golden_says() -> Result<(), Failed> {
    let drifted = Cases::<Transcript>::drifted(|case| {
        let directory = case.directory().to_path_buf();
        let transcript = case.into_row();
        match transcript.check {
            Check::Classified => {
                let run = Run::root();
                let port = run.port(ClaudeCodeDir::new(run.port(Disk), directory));
                let sessions = Transcripts::sessions(&port)?;
                let read = sessions
                    .iter()
                    .map(|session| Transcripts::read_from(&port, session, Cursor::default()))
                    .collect::<Result<Vec<Read>, _>>()?;
                drop(port);
                let snapshot = run.finish();
                let calls = [snapshot.calls::<Sessions>(), snapshot.calls::<ReadFrom>()];
                let stems: Vec<&SessionId> = sessions.iter().map(Session::stem).collect();
                Ok(Debugged((stems, calls, snapshot, read)).to_string())
            }
            Check::Resumed => {
                let reading = Run::root();
                let disk = reading.port(Disk);
                let session = disk.entries(&directory)?.into_iter().next().unwrap_or_default();
                let whole = disk.read_text(&session)?.as_ref().to_owned();
                reading.finish();
                let read = |text: String, cursor: Cursor| -> Result<Read, Failed> {
                    let files: Memory = [(session.clone(), Bytes::from(&text))].into_iter().collect();
                    let run = Run::root();
                    let port = run.port(ClaudeCodeDir::new(run.port(files), directory.clone()));
                    let sessions = Transcripts::sessions(&port)?;
                    let found = sessions.first().ok_or_else(|| Failed::from(session.display()))?;
                    let read = Transcripts::read_from(&port, found, cursor)?;
                    drop(port);
                    run.finish();
                    Ok(read)
                };
                let resumed = (usize::MIN..=whole.len())
                    .map(|cut| {
                        let first = read(whole.chars().take(cut).collect(), Cursor::default())?;
                        let second = read(whole.clone(), first.cursor())?;
                        let uuids: Vec<String> = first
                            .into_iter()
                            .chain(second)
                            .filter_map(|line| match line {
                                Line::Said(record) => Some(record.uuid().to_string()),
                                Line::Skipped(_) | Line::Unrecognised(_) => None,
                            })
                            .collect();
                        Ok(uuids)
                    })
                    .collect::<Result<BTreeSet<Vec<String>>, Failed>>()?;
                Ok(Debugged(resumed).to_string())
            }
            Check::Admitted => {
                let built: Vec<_> = transcript
                    .texts
                    .iter()
                    .map(|text| {
                        let owned = text.as_str().to_owned();
                        (
                            text,
                            Instant::try_new(owned.clone()).ignoring(Ignored::Refused).map(|made| made.to_string()),
                            Kind::try_new(owned.clone()).ignoring(Ignored::Refused).map(|made| made.to_string()),
                            SessionId::try_new(owned.clone()).ignoring(Ignored::Refused).map(|made| made.to_string()),
                            Body::try_new(owned).ignoring(Ignored::Refused).map(|made| made.to_string()),
                            text.as_str().parse::<Authorship>().ignoring(Ignored::Refused).map(|made| made.to_string()),
                        )
                    })
                    .collect();
                Ok(Debugged(built).to_string())
            }
        }
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}

proptest! {
    #![proptest_config(Properties::settled())]

    #[test]
    fn values_rebuild_from_their_display(instant in any::<Instant>(), named in any::<Kind>(), session in any::<SessionId>()) {
        prop_assert_eq!(Instant::try_new(instant.to_string()).ignoring(Ignored::Refused), Some(instant));
        prop_assert_eq!(Kind::try_new(named.to_string()).ignoring(Ignored::Refused), Some(named));
        prop_assert_eq!(SessionId::try_new(session.to_string()).ignoring(Ignored::Refused), Some(session));
    }
}
