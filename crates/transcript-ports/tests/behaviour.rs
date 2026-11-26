#![deny(clippy::wildcard_enum_match_arm)]

use std::path::{Path, PathBuf};
use test_failure::Failed;

use file_ports::{Bytes, Files, Memory};
use invariants::{Ignoring, ignored};
use port_core::{Calls, Run};
use proptest::prelude::{any, prop_assert_eq, proptest};
use proptest::test_runner::{Config, RngSeed};
use transcript_ports::transcripts_ops::{ReadFrom, Sessions};
use transcript_ports::{
    Authorship, ClaudeCodeDir, Cursor, Instant, Kind, Read, Record, Session, SessionId, Skipped, Transcripts,
    Unrecognised,
};

ignored! {
    enum Ignored {
        Unseeded => "seeding a fresh in-memory store does not fail; were it to, the read finds no session",
        Unread => "a read that fails yields nothing, which the case then reports",
        Refused => "a text the check or parse refuses is the None the case expects",
    }
}

const DIRECTORY: &str = "/projects/work";
const SESSION: &str = "/projects/work/s1.jsonl";

struct Lines;

impl Lines {
    fn entry(kind: &str, uuid: &str, origin: &str, content: &str) -> String {
        format!(
            "{{\"type\":\"{kind}\",\"uuid\":\"{uuid}\",\"parentUuid\":null,\"sessionId\":\"s1\",\"timestamp\":\"2026-01-01T00:00:0{uuid}Z\",\"origin\":{origin},\"message\":{{\"model\":\"claude\",\"content\":{content}}}}}\n"
        )
    }

    fn in_session(session: &str) -> String {
        format!(
            "{{\"type\":\"user\",\"uuid\":\"1\",\"parentUuid\":null,\"sessionId\":\"{session}\",\"timestamp\":\"2026-01-01T00:00:01Z\",\"origin\":{{\"kind\":\"human\"}},\"message\":{{\"content\":\"typed words\"}}}}\n"
        )
    }

    fn typed(uuid: &str) -> String {
        Self::entry("user", uuid, "{\"kind\":\"human\"}", "\"typed words\"")
    }

    fn answer(uuid: &str) -> String {
        Self::entry(
            "assistant",
            uuid,
            "null",
            "[{\"type\":\"text\",\"text\":\"first\"},{\"type\":\"tool_use\"},{\"type\":\"text\",\"text\":\"second\"}]",
        )
    }

    fn tool_result(uuid: &str) -> String {
        Self::entry("user", uuid, "null", "[{\"type\":\"tool_result\",\"content\":\"output\"}]")
    }

    fn notice(uuid: &str) -> String {
        Self::entry("user", uuid, "{\"kind\":\"task-notification\"}", "\"finished\"")
    }

    fn shaped(kind: &str, fields: &str) -> String {
        format!(
            "{{\"type\":\"{kind}\",\"uuid\":\"8\",\"parentUuid\":\"1\",\"sessionId\":\"s1\",\"timestamp\":\"2026-01-01T00:00:08Z\",{fields}}}\n"
        )
    }

    fn system(uuid: &str, parent: &str) -> String {
        format!(
            "{{\"type\":\"system\",\"subtype\":\"turn_duration\",\"uuid\":\"{uuid}\",\"parentUuid\":\"{parent}\",\"sessionId\":\"s1\"}}\n"
        )
    }

    fn skipped() -> String {
        "{\"type\":\"permission-mode\",\"sessionId\":\"s1\"}\n".to_owned()
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Counted {
    sessions: Calls,
    reads: Calls,
}

struct Transcript(String);

impl Transcript {
    fn holding(text: &str) -> Transcript {
        Transcript(text.to_owned())
    }

    fn directory(&self) -> ClaudeCodeDir<Memory> {
        let memory = Memory::default();
        memory.create_directories(Path::new(DIRECTORY)).ignoring(Ignored::Unseeded);
        memory.append(Path::new(SESSION), &Bytes::from(&self.0)).ignoring(Ignored::Unseeded);
        ClaudeCodeDir::new(memory, PathBuf::from(DIRECTORY))
    }

    fn read(&self, cursor: Cursor) -> Result<Read, Failed> {
        let run = Run::root();
        let port = run.port(self.directory());
        let sessions = Transcripts::sessions(&port)?;
        let session = sessions.first().ok_or("no session")?;
        Ok(Transcripts::read_from(&port, session, cursor)?)
    }

    fn sessions(&self) -> Vec<Session> {
        Transcripts::sessions(&Run::root().port(self.directory())).ignoring(Ignored::Unread).unwrap_or_default()
    }

    fn calls(&self) -> Option<Counted> {
        let run = Run::root();
        let port = run.port(self.directory());
        let sessions = Transcripts::sessions(&port).ignoring(Ignored::Unread)?;
        sessions
            .iter()
            .map(|session| Transcripts::read_from(&port, session, Cursor::default()))
            .collect::<Result<Vec<_>, _>>()
            .ignoring(Ignored::Unread)?;
        drop(port);
        let measured = run.finish();
        Some(Counted {
            sessions: measured.snapshot().calls::<Sessions>(),
            reads: measured.snapshot().calls::<ReadFrom>(),
        })
    }
}

#[derive(Default)]
struct Gathered {
    records: Vec<Record>,
    skipped: Vec<Skipped>,
    classes: Vec<String>,
}

impl From<Record> for Gathered {
    fn from(record: Record) -> Gathered {
        let classes = vec![format!("{:?} {}", record.author(), record.text())];
        Gathered { records: vec![record], skipped: Vec::new(), classes }
    }
}

impl From<Skipped> for Gathered {
    fn from(skipped: Skipped) -> Gathered {
        Gathered {
            records: Vec::new(),
            classes: Gathered::edge(&skipped).into_iter().collect(),
            skipped: vec![skipped],
        }
    }
}

impl From<Unrecognised> for Gathered {
    fn from(unrecognised: Unrecognised) -> Gathered {
        let kind = format!("unrecognised {}", unrecognised.kind());
        let classes = [kind].into_iter().chain(unrecognised.passing().as_ref().and_then(Gathered::edge)).collect();
        Gathered { records: Vec::new(), skipped: Vec::new(), classes }
    }
}

impl Gathered {
    fn joined(self, later: Gathered) -> Gathered {
        Gathered {
            records: self.records.into_iter().chain(later.records).collect(),
            skipped: self.skipped.into_iter().chain(later.skipped).collect(),
            classes: self.classes.into_iter().chain(later.classes).collect(),
        }
    }

    fn edge(skipped: &Skipped) -> Option<String> {
        skipped.parent().map(|parent| format!("{} after {parent}", skipped.uuid()))
    }
}

impl From<Read> for Gathered {
    fn from(read: Read) -> Gathered {
        read.into_iter().fold(Gathered::default(), |gathered, line| gathered.joined(line.gathered()))
    }
}

struct Cases;

impl Cases {
    fn classified(line: &str) -> Vec<String> {
        Transcript::holding(line)
            .read(Cursor::default())
            .ignoring(Ignored::Unread)
            .map_or_default(|read| Gathered::from(read).classes)
    }

    fn kind(kind: &&str) -> Vec<String> {
        Self::classified(&Lines::shaped(kind, "\"slug\":\"quiet\""))
    }

    fn user(fields: &&str) -> Vec<String> {
        Self::classified(&Lines::shaped("user", fields))
    }

    fn assistant(fields: &&str) -> Vec<String> {
        Self::classified(&Lines::shaped("assistant", fields))
    }

    fn attached(attachment: &&str) -> Vec<String> {
        Self::classified(&Lines::shaped("attachment", &format!("\"attachment\":{attachment}")))
    }

    fn carried(kind: &&str) -> Vec<String> {
        Self::attached(&format!("{{\"type\":\"{kind}\",\"content\":\"ok\"}}").as_str())
    }
}

struct Views;

impl Views {
    fn records(read: Read) -> Vec<Record> {
        Gathered::from(read).records
    }

    fn edges(read: Read) -> Vec<String> {
        Gathered::from(read)
            .skipped
            .into_iter()
            .filter_map(|skipped| skipped.parent().map(|parent| format!("{} after {parent}", skipped.uuid())))
            .collect()
    }

    fn authors(read: Read) -> Vec<Authorship> {
        Self::records(read).into_iter().map(|record| record.author()).collect()
    }

    fn uuids(records: &[Record]) -> Vec<String> {
        records.iter().map(|record| record.uuid().to_string()).collect()
    }

    fn texts(read: Read) -> Vec<String> {
        Self::records(read).into_iter().map(|record| record.text().to_string()).collect()
    }
}

#[test]
fn each_line_is_classified_by_who_wrote_it() -> Result<(), Failed> {
    let text =
        [Lines::typed("1"), Lines::answer("2"), Lines::tool_result("3"), Lines::notice("4"), Lines::skipped()].concat();
    let read = Transcript::holding(&text).read(Cursor::default())?;
    assert_eq!(Views::authors(read), [Authorship::Owner, Authorship::Lead]);
    Ok(())
}

#[test]
fn a_line_that_is_not_a_message_yields_its_parent_edge() -> Result<(), Failed> {
    let text = [Lines::answer("2"), Lines::system("5", "2"), Lines::skipped()].concat();
    let read = Transcript::holding(&text).read(Cursor::default())?;
    assert_eq!(Views::edges(read), ["5 after 2"]);
    Ok(())
}

#[test]
fn a_lead_record_keeps_only_its_text_blocks() -> Result<(), Failed> {
    let read = Transcript::holding(&Lines::answer("2")).read(Cursor::default())?;
    assert_eq!(Views::texts(read), ["first second"]);
    Ok(())
}

#[test]
fn a_record_carries_the_timestamp_of_its_line() -> Result<(), Failed> {
    let read = Transcript::holding(&Lines::typed("1")).read(Cursor::default())?;
    let stamps: Vec<Instant> = Views::records(read).into_iter().map(|record| record.stamp().clone()).collect();
    assert_eq!(stamps, [Instant::try_new("2026-01-01T00:00:01Z".to_owned())?]);
    Ok(())
}

#[test]
fn a_record_carries_its_session_and_parent() -> Result<(), Failed> {
    let read = Transcript::holding(&Lines::shaped(
        "assistant",
        r#""message":{"model":"claude","content":[{"type":"text","text":"a"}]}"#,
    ))
    .read(Cursor::default())?;
    let found: Vec<String> = Views::records(read)
        .into_iter()
        .map(|record| format!("{} {:?}", record.session(), record.parent().map(ToString::to_string)))
        .collect();
    assert_eq!(found, ["s1 Some(\"1\")"]);
    Ok(())
}

#[test]
fn a_partial_last_line_is_not_yielded() -> Result<(), Failed> {
    let whole = Lines::typed("1");
    let partial = Lines::answer("2");
    let cut = partial.chars().take(partial.len() / 2).collect::<String>();
    let read = Transcript::holding(&format!("{whole}{cut}")).read(Cursor::default())?;
    let records = Views::records(read);
    assert_eq!(Views::uuids(&records), ["1"]);
    Ok(())
}

#[test]
fn a_record_uuid_reads_as_its_text() -> Result<(), Failed> {
    let read = Transcript::holding(&Lines::typed("abc")).read(Cursor::default())?;
    let texts: Vec<String> = Views::records(read).into_iter().map(|record| record.uuid().to_string()).collect();
    assert_eq!(texts, ["abc"]);
    Ok(())
}

impl Views {
    fn uuids_of(uuid: &str) -> Option<Vec<String>> {
        Transcript::holding(&Lines::typed(uuid)).read(Cursor::default()).ignoring(Ignored::Unread).map(|read| {
            Views::records(read).into_iter().map(|record| record.uuid().to_string()).collect::<Vec<String>>()
        })
    }

    fn sessions_of(session: &str) -> Option<Vec<String>> {
        Transcript::holding(&Lines::in_session(session)).read(Cursor::default()).ignoring(Ignored::Unread).map(|read| {
            Views::records(read).into_iter().map(|record| record.session().to_string()).collect::<Vec<String>>()
        })
    }
}

#[test]
fn a_message_uuid_is_read_only_when_it_is_one_bare_word() {
    assert_eq!(Views::uuids_of("abc"), Some(vec![String::from("abc")]));
    assert_eq!(Views::uuids_of("_a-9"), Some(vec![String::from("_a-9")]));
    for uuid in ["msg:1", "a b", "-a", ""] {
        assert_eq!(Views::uuids_of(uuid), Some(Vec::new()), "{uuid}");
    }
}

#[test]
fn a_line_whose_session_is_blank_yields_no_record() {
    assert_eq!(Views::sessions_of("s1"), Some(vec![String::from("s1")]));
    assert_eq!(Views::sessions_of("a b"), Some(vec![String::from("a b")]));
    assert_eq!(Views::sessions_of(""), Some(Vec::new()));
}

#[test]
fn a_session_is_listed_by_its_file_stem() {
    let sessions = Transcript::holding(&Lines::typed("1")).sessions();
    assert_eq!(sessions.iter().map(|session| session.stem().to_string()).collect::<Vec<_>>(), ["s1"]);
}

#[test]
fn reading_all_sessions_counts_one_listing_and_one_read_per_session() {
    let counted = Transcript::holding(&Lines::typed("1")).calls();
    assert_eq!(counted, Some(Counted { sessions: Calls::new(1), reads: Calls::new(1) }));
}

#[test]
fn resuming_from_the_cursor_yields_every_record_exactly_once() -> Result<(), Failed> {
    let lines = [Lines::typed("1"), Lines::answer("2"), Lines::tool_result("3"), Lines::notice("4")];
    let text = lines.concat();
    (0..=text.len()).try_for_each(|cut| {
        let first = Transcript::holding(&text.chars().take(cut).collect::<String>()).read(Cursor::default())?;
        let second = Transcript::holding(&text).read(first.cursor())?;
        let together: Vec<Record> = Views::records(first).into_iter().chain(Views::records(second)).collect();
        assert_eq!(Views::uuids(&together), ["1", "2"], "cut at {cut}");
        Ok(())
    })
}

proptest! {
    #![proptest_config(Config { failure_persistence: None, rng_seed: RngSeed::Fixed(0), ..Config::default() })]
    #[test]
    fn an_unknown_record_kind_is_unrecognised_and_never_a_message(kind in "[a-z]{1,8}-unheard") {
        let line = Lines::shaped(&kind, r#""origin":{"kind":"human"},"message":{"model":"claude","content":"hi"}"#);
        prop_assert_eq!(Cases::classified(&line), [format!("unrecognised {kind}"), "8 after 1".to_owned()]);
    }

    #[test]
    fn an_unknown_block_in_a_lead_record_is_unrecognised(block in "[a-z]{1,8}_unheard") {
        let fields =
            format!(r#""message":{{"model":"claude","content":[{{"type":"text","text":"said"}},{{"type":"{block}"}}]}}"#);
        prop_assert_eq!(Cases::assistant(&fields.as_str()), [format!("unrecognised {block}"), "8 after 1".to_owned()]);
    }

    #[test]
    fn an_instant_is_rebuilt_from_its_display(instant in any::<Instant>()) {
        prop_assert_eq!(Instant::try_new(instant.to_string()).ignoring(Ignored::Refused), Some(instant));
    }

    #[test]
    fn a_kind_is_rebuilt_from_its_display(kind in any::<Kind>()) {
        prop_assert_eq!(Kind::try_new(kind.to_string()).ignoring(Ignored::Refused), Some(kind));
    }

    #[test]
    fn a_session_id_is_rebuilt_from_its_display(session in any::<SessionId>()) {
        prop_assert_eq!(SessionId::try_new(session.to_string()).ignoring(Ignored::Refused), Some(session));
    }
}

impl Cases {
    fn parents(texts: &[&str], read: fn(&&str) -> Vec<String>) {
        for text in texts {
            assert_eq!(read(text), ["8 after 1"], "{text}");
        }
    }
}

#[test]
fn every_surveyed_record_kind_without_a_voice_is_only_a_parent_edge() {
    assert_eq!(Cases::kind(&"brand-new-kind"), ["unrecognised brand-new-kind", "8 after 1"]);
    Cases::parents(
        &[
            "ai-title",
            "api-request",
            "api-request-blob",
            "api-request-shape",
            "artifact-autoreact-ledger",
            "artifact-comment-monitor",
            "atis-latch",
            "cost-state",
            "file-history-delta",
            "file-history-snapshot",
            "frame-link",
            "last-prompt",
            "launched",
            "mode",
            "permission-mode",
            "pr-link",
            "queue-operation",
            "result",
            "started",
            "summary",
            "system",
            "tools-used",
        ],
        Cases::kind,
    );
}

#[test]
fn a_user_record_is_the_owner_only_when_a_human_typed_text_in_the_main_chain() {
    assert_eq!(Cases::user(&r#""origin":{"kind":"human"},"message":{"content":"typed"}"#), ["Owner typed"]);
    assert_eq!(
        Cases::user(
            &r#""origin":{"kind":"human"},"message":{"content":[{"type":"image","source":{}},{"type":"text","text":"look"}]}"#
        ),
        ["Owner look"]
    );
    assert_eq!(
        Cases::user(&r#""origin":{"kind":"brand-new-kind"},"message":{"content":"hello"}"#),
        ["unrecognised brand-new-kind", "8 after 1"]
    );
    assert_eq!(
        Cases::user(
            &r#""origin":{"kind":"brand-new-kind"},"message":{"content":[{"type":"tool_result","content":"out"}]}"#
        ),
        ["unrecognised brand-new-kind", "8 after 1"]
    );
    assert_eq!(
        Cases::user(
            &r#""origin":{"kind":"human"},"message":{"content":[{"type":"document"},{"type":"text","text":"a"}]}"#
        ),
        ["unrecognised document", "8 after 1"]
    );
    Cases::parents(
        &[
            r#""origin":{"kind":"human"},"message":{"content":[{"type":"tool_result","content":"out"}]}"#,
            r#""origin":{"kind":"human"},"isSidechain":true,"isMeta":true,"message":{"content":"relayed"}"#,
            r#""origin":{"kind":"task-notification"},"message":{"content":"finished"}"#,
            r#""origin":{"kind":"task-notification"},"isSidechain":true,"isMeta":true,"message":{"content":"finished"}"#,
            r#""origin":{"kind":"coordinator"},"isSidechain":true,"isMeta":true,"message":{"content":"go"}"#,
            r#""origin":{"kind":"peer"},"isMeta":true,"message":{"content":"hello"}"#,
            r#""origin":{"kind":"peer"},"isSidechain":true,"isMeta":true,"message":{"content":"hello"}"#,
            r#""origin":{"kind":"unclassified"},"isSidechain":true,"message":{"content":[{"type":"tool_result","content":"out"}]}"#,
            r#""origin":{"kind":"unclassified"},"isSidechain":true,"message":{"content":"brief"}"#,
            r#""origin":{"kind":"auto-continuation"},"isMeta":true,"message":{"content":"continue"}"#,
            r#""origin":null,"message":{"content":[{"type":"tool_result","content":"out"}]}"#,
            r#""isSidechain":true,"message":{"content":[{"type":"tool_result","content":"out"}]}"#,
            r#""message":{"content":"<command-name>/compact</command-name>"}"#,
            r#""isMeta":true,"message":{"content":"caveat"}"#,
            r#""isSidechain":true,"message":{"content":"brief"}"#,
            r#""isSidechain":true,"isMeta":true,"message":{"content":"brief"}"#,
            r#""message":{"content":[{"type":"text","text":"[Request interrupted by user]"}]}"#,
            r#""isMeta":true,"message":{"content":[{"type":"text","text":"skill"}]}"#,
            r#""isSidechain":true,"message":{"content":[{"type":"text","text":"a"},{"type":"tool_result","content":"b"}]}"#,
            r#""isCompactSummary":true,"isVisibleInTranscriptOnly":true,"message":{"content":"summary"}"#,
        ],
        Cases::user,
    );
}

#[test]
fn an_assistant_record_is_the_lead_only_when_it_answered_with_text_in_the_main_chain() {
    assert_eq!(
        Cases::assistant(&r#""message":{"model":"claude","content":[{"type":"text","text":"said"}]}"#),
        ["Lead said"]
    );
    assert_eq!(
        Cases::assistant(&r#""message":{"model":"claude","content":[{"type":"server_tool_use"}]}"#),
        ["unrecognised server_tool_use", "8 after 1"]
    );
    Cases::parents(
        &[
            r#""message":{"model":"claude","content":[{"type":"thinking","thinking":"hmm"}]}"#,
            r#""message":{"model":"claude","content":[{"type":"tool_use","name":"Bash"}]}"#,
            r#""isSidechain":true,"message":{"model":"claude","content":[{"type":"text","text":"subagent words"}]}"#,
            r#""isSidechain":true,"message":{"model":"claude","content":[{"type":"thinking","thinking":"hmm"}]}"#,
            r#""isSidechain":true,"message":{"model":"claude","content":[{"type":"tool_use","name":"Bash"}]}"#,
            r#""isApiErrorMessage":true,"message":{"model":"<synthetic>","content":[{"type":"text","text":"API Error"}]}"#,
            r#""isApiErrorMessage":true,"message":{"model":"claude","content":[{"type":"text","text":"API Error"}]}"#,
            r#""isSidechain":true,"isApiErrorMessage":true,"message":{"model":"<synthetic>","content":[{"type":"text","text":"API Error"}]}"#,
            r#""message":{"model":"<synthetic>","content":[{"type":"text","text":"No response requested."}]}"#,
        ],
        Cases::assistant,
    );
}

#[test]
fn a_queued_command_is_the_owner_only_when_a_human_queued_it() {
    assert_eq!(
        Cases::attached(&r#"{"type":"queued_command","prompt":"typed while busy","origin":{"kind":"human"}}"#),
        ["Owner typed while busy"]
    );
    assert_eq!(
        Cases::attached(
            &r#"{"type":"queued_command","prompt":[{"type":"text","text":"see"},{"type":"image"}],"origin":{"kind":"human"}}"#
        ),
        ["Owner see"]
    );
    assert_eq!(
        Cases::attached(&r#"{"type":"queued_command","prompt":"hello","origin":{"kind":"brand-new-kind"}}"#),
        ["unrecognised brand-new-kind", "8 after 1"]
    );
    assert_eq!(
        Cases::attached(&r#"{"type":"brand_new_attachment"}"#),
        ["unrecognised brand_new_attachment", "8 after 1"]
    );
    Cases::parents(
        &[
            r#"{"type":"queued_command","prompt":"finished","origin":{"kind":"task-notification"}}"#,
            r#"{"type":"queued_command","prompt":"go","origin":{"kind":"coordinator"}}"#,
            r#"{"type":"queued_command","prompt":"hello","origin":{"kind":"peer"}}"#,
            r#"{"type":"queued_command","prompt":"finished"}"#,
        ],
        Cases::attached,
    );
}

#[test]
fn every_surveyed_attachment_other_than_a_queued_command_is_only_a_parent_edge() {
    Cases::parents(
        &[
            "agent_listing_delta",
            "auto_mode",
            "bash_output_audience_note",
            "batching_reminder_sent",
            "command_permissions",
            "compact_file_reference",
            "credential_org",
            "date",
            "deferred_tools_delta",
            "deferred_tools_record",
            "edited_text_file",
            "environment",
            "file",
            "hook_additional_context",
            "hook_blocking_error",
            "hook_non_blocking_error",
            "hook_permission_decision",
            "hook_success",
            "hook_system_message",
            "instructions",
            "invoked_skills",
            "mcp_instructions_delta",
            "model",
            "nested_memory",
            "prompt_snapshot",
            "read_truncation_notice",
            "remote_session_change",
            "session_context",
            "session_cron_carry",
            "silent_turn_reminder",
            "skill_listing",
            "task_reminder",
            "task_status",
            "thinking_drop",
            "total_tokens_reminder",
            "ultrathink_effort",
        ],
        Cases::carried,
    );
}

#[test]
fn an_authorship_is_read_back_from_its_display() {
    for author in [Authorship::Owner, Authorship::Lead] {
        assert_eq!(author.to_string().parse::<Authorship>().ignoring(Ignored::Refused), Some(author));
    }
}

#[test]
fn a_name_of_neither_party_is_no_authorship() {
    for name in ["human", ""] {
        assert_eq!(name.parse::<Authorship>().ignoring(Ignored::Refused), None);
    }
}

#[test]
fn blank_text_makes_no_instant_kind_or_session_id() {
    assert_eq!(Instant::try_new(String::new()).ignoring(Ignored::Refused), None);
    assert_eq!(Kind::try_new(String::new()).ignoring(Ignored::Refused), None);
    assert_eq!(SessionId::try_new(String::new()).ignoring(Ignored::Refused), None);
    let instant = Instant::try_new(String::from("2026-01-01T00:00:01Z")).ignoring(Ignored::Refused);
    assert_eq!(instant.map(|made| made.to_string()).as_deref(), Some("2026-01-01T00:00:01Z"));
    let kind = Kind::try_new(String::from("summary")).ignoring(Ignored::Refused);
    assert_eq!(kind.map(|made| made.to_string()).as_deref(), Some("summary"));
    let session = SessionId::try_new(String::from("s1")).ignoring(Ignored::Refused);
    assert_eq!(session.map(|made| made.to_string()).as_deref(), Some("s1"));
}

#[test]
fn a_body_holding_a_nul_is_refused_and_any_other_text_is_kept_whole() {
    let read = |text: &str| transcript_ports::Body::try_new(text.to_owned()).ignoring(Ignored::Refused);
    assert_eq!(read("said\0plainly"), None);
    assert_eq!(read("said plainly").map(|body| body.to_string()).as_deref(), Some("said plainly"));
}
