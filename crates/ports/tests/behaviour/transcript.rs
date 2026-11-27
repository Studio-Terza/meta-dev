#![deny(clippy::wildcard_enum_match_arm)]

use std::path::{Path, PathBuf};
use test_failure::Failed;

use file_ports::{Bytes, Disk, Files, Memory};
use grammar::KeptLines;
use invariants::{Ignoring, ignored};
use ports::port::{Calls, Run};
use ports::transcript::transcripts_ops::{ReadFrom, Sessions};
use ports::transcript::{
    Authorship, Body, ClaudeCodeDir, Cursor, Instant, Kind, Line, Read, Record, SessionId, Skipped, Transcripts,
    Unrecognised,
};
use proptest::prelude::{any, prop_assert_eq, proptest};
use proptest::test_runner::{Config, RngSeed};

ignored! {
    enum Ignored {
        Unseeded => "seeding a fresh in-memory store does not fail; were it to, the read finds no session",
        Refused => "a text the check or parse refuses is the None the case expects",
    }
}

const DIRECTORY: &str = "/projects/work";
const SESSION: &str = "/projects/work/s1.jsonl";
const FIXTURE: &str = "tests/behaviour/lines.jsonl";
const GOLDEN: &str = "tests/behaviour/lines.txt";
const RECORD: &str = "RECORD_GOLDENS";

struct Transcript(String);

impl Transcript {
    fn fixture() -> Result<String, Failed> {
        Ok(Disk.read_text(&Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE))?.as_ref().to_owned())
    }

    fn read(&self, cursor: Cursor) -> Result<Read, Failed> {
        let memory = Memory::default();
        memory.create_directories(Path::new(DIRECTORY)).ignoring(Ignored::Unseeded);
        memory.append(Path::new(SESSION), &Bytes::from(&self.0)).ignoring(Ignored::Unseeded);
        let run = Run::root();
        let port = run.port(ClaudeCodeDir::new(memory, PathBuf::from(DIRECTORY)));
        let sessions = Transcripts::sessions(&port)?;
        let read = Transcripts::read_from(&port, sessions.first().ok_or("no session")?, cursor)?;
        drop(port);
        let measured = run.finish();
        let calls = [measured.snapshot().calls::<Sessions>(), measured.snapshot().calls::<ReadFrom>()];
        assert_eq!(
            (sessions.first().map(|session| session.stem().to_string()), calls),
            (Some("s1".to_owned()), [Calls::new(1); 2])
        );
        measured.settled()?;
        Ok(read)
    }

    fn shown(line: Line) -> Vec<String> {
        let edge = |skipped: &Skipped| skipped.parent().map(|above| format!("{} after {above}", skipped.uuid()));
        match line {
            Line::Said(record) => vec![Transcript::said(&record)],
            Line::Skipped(skipped) => edge(&skipped).into_iter().collect(),
            Line::Unrecognised(unrecognised) => {
                let unknown: &Unrecognised = &unrecognised;
                let passing = unknown.passing().as_ref().and_then(edge);
                [format!("unrecognised {}", unknown.kind())].into_iter().chain(passing).collect()
            }
        }
    }

    fn said(record: &Record) -> String {
        let parent = record.parent().map(ToString::to_string);
        format!(
            "{:?} {} [{} in {} after {parent:?} at {}]",
            record.author(),
            record.text(),
            record.uuid(),
            record.session(),
            record.stamp()
        )
    }

    fn uuids(read: Read) -> Vec<String> {
        read.into_iter()
            .filter_map(|line| match line {
                Line::Said(record) => Some(record.uuid().to_string()),
                Line::Skipped(_) | Line::Unrecognised(_) => None,
            })
            .collect()
    }
}

#[test]
fn every_fixture_line_reads_as_its_golden_classification() -> Result<(), Failed> {
    let read = Transcript(Transcript::fixture()?).read(Cursor::default())?;
    let shown: String =
        read.into_iter().flat_map(Transcript::shown).flat_map(|class| [class, "\n".to_owned()]).collect();
    let golden = Path::new(env!("CARGO_MANIFEST_DIR")).join(GOLDEN);
    std::env::var_os(RECORD).map_or(Ok(()), |_| Disk.write(&golden, Bytes::new(shown.clone().into_bytes())))?;
    assert_eq!(shown, Disk.read_text(&golden)?.as_ref());
    Ok(())
}

#[test]
fn resuming_from_the_cursor_yields_every_record_exactly_once() -> Result<(), Failed> {
    let lines: Vec<String> = Transcript::fixture()?.parse::<KeptLines>().into_iter().flatten().take(4).collect();
    let text = lines.concat();
    (0..=text.len()).try_for_each(|cut| {
        let first = Transcript(text.chars().take(cut).collect()).read(Cursor::default())?;
        let second = Transcript(text.clone()).read(first.cursor())?;
        let together: Vec<String> = Transcript::uuids(first).into_iter().chain(Transcript::uuids(second)).collect();
        assert_eq!(together, ["1", "2"], "cut at {cut}");
        Ok(())
    })
}

proptest! {
    #![proptest_config(Config { failure_persistence: None, rng_seed: RngSeed::Fixed(0), ..Config::default() })]
    #[test]
    fn unknown_kinds_and_blocks_are_unrecognised_and_values_rebuild_from_their_display(kind in "[a-z]{1,8}-unheard", block in "[a-z]{1,8}_unheard", instant in any::<Instant>(), named in any::<Kind>(), session in any::<SessionId>()) {
        let head = "\"uuid\":\"8\",\"parentUuid\":\"1\",\"sessionId\":\"s1\",\"timestamp\":\"2026-01-01T00:00:08Z\"";
        let unknown = format!("{{\"type\":\"{kind}\",{head},\"message\":{{\"content\":\"hi\"}}}}\n");
        let blocked = format!("{{\"type\":\"assistant\",{head},\"message\":{{\"model\":\"claude\",\"content\":[{{\"type\":\"{block}\"}}]}}}}\n");
        let shown = |text: String| Transcript(text).read(Cursor::default()).map(|read| read.into_iter().flat_map(Transcript::shown).collect::<Vec<String>>());
        prop_assert_eq!(shown(unknown), Ok(vec![format!("unrecognised {kind}"), "8 after 1".to_owned()]));
        prop_assert_eq!(shown(blocked), Ok(vec![format!("unrecognised {block}"), "8 after 1".to_owned()]));
        prop_assert_eq!(Instant::try_new(instant.to_string()).ignoring(Ignored::Refused), Some(instant));
        prop_assert_eq!(Kind::try_new(named.to_string()).ignoring(Ignored::Refused), Some(named));
        prop_assert_eq!(SessionId::try_new(session.to_string()).ignoring(Ignored::Refused), Some(session));
    }
}

#[test]
fn values_are_built_only_from_the_texts_they_admit() {
    let built = |text: &str| {
        [
            Instant::try_new(text.to_owned()).ignoring(Ignored::Refused).map(|made| made.to_string()),
            Kind::try_new(text.to_owned()).ignoring(Ignored::Refused).map(|made| made.to_string()),
            SessionId::try_new(text.to_owned()).ignoring(Ignored::Refused).map(|made| made.to_string()),
            Body::try_new(text.to_owned()).ignoring(Ignored::Refused).map(|made| made.to_string()),
            text.parse::<Authorship>().ignoring(Ignored::Refused).map(|made| made.to_string()),
        ]
    };
    let all = |text: &str, admitted: [u8; 5]| admitted.map(|admits| (admits == 1).then(|| text.to_owned()));
    assert_eq!(built(""), all("", [0, 0, 0, 1, 0]));
    assert_eq!(built("s1"), all("s1", [1, 1, 1, 1, 0]));
    assert_eq!(built("said\0plainly"), all("said\0plainly", [1, 1, 1, 0, 0]));
    assert_eq!([built("owner"), built("lead")], [all("owner", [1; 5]), all("lead", [1; 5])]);
}
