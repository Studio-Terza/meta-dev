#![deny(clippy::wildcard_enum_match_arm)]

use std::error::Error;
use std::path::{Path, PathBuf};

use file_ports::{Bytes, Files, Memory};
use laws::proptest::prelude::{any, prop_assert_eq};
use laws::proptest::sample::select;
use laws::proptest::test_runner::TestCaseError;
use port_core::{Calls, Run};
use transcript_ports::transcripts_ops::{ReadFrom, Sessions};
use transcript_ports::{
    Authorship, ClaudeCodeDir, Cursor, Gather, Instant, Kind, Read, Record, Session, SessionId, Skipped, Transcripts,
    Unrecognised,
};

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
        memory.create_directories(Path::new(DIRECTORY)).ok();
        memory.append(Path::new(SESSION), &Bytes::from(&self.0)).ok();
        ClaudeCodeDir::new(memory, PathBuf::from(DIRECTORY))
    }

    fn read(&self, cursor: Cursor) -> Result<Read, Box<dyn Error>> {
        let run = Run::root();
        let port = run.port(self.directory());
        let sessions = Transcripts::sessions(&port)?;
        let session = sessions.first().ok_or("no session")?;
        Ok(Transcripts::read_from(&port, session, cursor)?)
    }

    fn sessions(&self) -> Vec<Session> {
        Transcripts::sessions(&Run::root().port(self.directory())).unwrap_or_default()
    }

    fn calls(&self) -> Option<Counted> {
        let run = Run::root();
        let port = run.port(self.directory());
        let sessions = Transcripts::sessions(&port).ok()?;
        sessions
            .iter()
            .map(|session| Transcripts::read_from(&port, session, Cursor::default()))
            .collect::<Result<Vec<_>, _>>()
            .ok()?;
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

impl Gather for Gathered {
    fn said(mut self, record: Record) -> Gathered {
        self.classes.push(format!("{:?} {}", record.author(), record.text()));
        self.records.push(record);
        self
    }

    fn skipped(mut self, skipped: Skipped) -> Gathered {
        self.classes.extend(Gathered::edge(&skipped));
        self.skipped.push(skipped);
        self
    }

    fn unrecognised(mut self, unrecognised: Unrecognised) -> Gathered {
        self.classes.push(format!("unrecognised {}", unrecognised.kind()));
        self.classes.extend(unrecognised.passing().as_ref().and_then(Gathered::edge));
        self
    }
}

impl Gathered {
    fn edge(skipped: &Skipped) -> Option<String> {
        skipped.parent().map(|parent| format!("{} after {parent}", skipped.uuid()))
    }
}

impl From<Read> for Gathered {
    fn from(read: Read) -> Gathered {
        read.into_iter().fold(Gathered::default(), |gathered, line| line.gathered(gathered))
    }
}

struct Cases;

impl Cases {
    fn classified(line: &str) -> Vec<String> {
        Transcript::holding(line).read(Cursor::default()).map(|read| Gathered::from(read).classes).unwrap_or_default()
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
fn each_line_is_classified_by_who_wrote_it() -> Result<(), Box<dyn Error>> {
    let text =
        [Lines::typed("1"), Lines::answer("2"), Lines::tool_result("3"), Lines::notice("4"), Lines::skipped()].concat();
    let read = Transcript::holding(&text).read(Cursor::default())?;
    assert_eq!(Views::authors(read), [Authorship::Owner, Authorship::Lead]);
    Ok(())
}

#[test]
fn a_line_that_is_not_a_message_yields_its_parent_edge() -> Result<(), Box<dyn Error>> {
    let text = [Lines::answer("2"), Lines::system("5", "2"), Lines::skipped()].concat();
    let read = Transcript::holding(&text).read(Cursor::default())?;
    assert_eq!(Views::edges(read), ["5 after 2"]);
    Ok(())
}

#[test]
fn a_lead_record_keeps_only_its_text_blocks() -> Result<(), Box<dyn Error>> {
    let read = Transcript::holding(&Lines::answer("2")).read(Cursor::default())?;
    assert_eq!(Views::texts(read), ["first second"]);
    Ok(())
}

#[test]
fn a_record_carries_the_timestamp_of_its_line() -> Result<(), Box<dyn Error>> {
    let read = Transcript::holding(&Lines::typed("1")).read(Cursor::default())?;
    let stamps: Vec<Instant> = Views::records(read).into_iter().map(|record| record.stamp().clone()).collect();
    assert_eq!(stamps, [Instant::try_new("2026-01-01T00:00:01Z".to_owned())?]);
    Ok(())
}

#[test]
fn a_record_carries_its_session_and_parent() -> Result<(), Box<dyn Error>> {
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
fn a_partial_last_line_is_not_yielded() -> Result<(), Box<dyn Error>> {
    let whole = Lines::typed("1");
    let partial = Lines::answer("2");
    let cut = partial.chars().take(partial.len() / 2).collect::<String>();
    let read = Transcript::holding(&format!("{whole}{cut}")).read(Cursor::default())?;
    let records = Views::records(read);
    assert_eq!(Views::uuids(&records), ["1"]);
    Ok(())
}

#[test]
fn a_record_uuid_reads_as_its_text() -> Result<(), Box<dyn Error>> {
    let read = Transcript::holding(&Lines::typed("abc")).read(Cursor::default())?;
    let texts: Vec<String> = Views::records(read).into_iter().map(|record| record.uuid().to_string()).collect();
    assert_eq!(texts, ["abc"]);
    Ok(())
}

laws::table!(
    a_message_uuid_is_read_only_when_it_is_one_bare_word,
    |uuid: &&str| {
        Transcript::holding(&Lines::typed(uuid)).read(Cursor::default()).ok().map(|read| {
            Views::records(read).into_iter().map(|record| record.uuid().to_string()).collect::<Vec<String>>()
        })
    },
    [
        "abc" => Some(vec![String::from("abc")]),
        "_a-9" => Some(vec![String::from("_a-9")]),
        "msg:1" => None,
        "a b" => None,
        "-a" => None,
        "" => None,
    ]
);

laws::table!(
    a_line_whose_session_is_blank_yields_no_record,
    |session: &&str| {
        Transcript::holding(&Lines::in_session(session)).read(Cursor::default()).ok().map(|read| {
            Views::records(read).into_iter().map(|record| record.session().to_string()).collect::<Vec<String>>()
        })
    },
    [
        "s1" => Some(vec![String::from("s1")]),
        "a b" => Some(vec![String::from("a b")]),
        "" => Some(Vec::new()),
    ]
);

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

laws::law!(resuming_from_the_cursor_yields_every_record_exactly_once, 0usize..400, split, {
    let lines = [Lines::typed("1"), Lines::answer("2"), Lines::tool_result("3"), Lines::notice("4")];
    let text = lines.concat();
    let cut = split.min(text.len());
    let first = Transcript::holding(&text.chars().take(cut).collect::<String>())
        .read(Cursor::default())
        .map_err(|failure| TestCaseError::fail(failure.to_string()))?;
    let second =
        Transcript::holding(&text).read(first.cursor()).map_err(|failure| TestCaseError::fail(failure.to_string()))?;
    let together: Vec<Record> = Views::records(first).into_iter().chain(Views::records(second)).collect();
    prop_assert_eq!(Views::uuids(&together), ["1", "2"]);
});

laws::law!(an_unknown_record_kind_is_unrecognised_and_never_a_message, "[a-z]{1,8}-unheard", kind, {
    let line = Lines::shaped(&kind, r#""origin":{"kind":"human"},"message":{"model":"claude","content":"hi"}"#);
    prop_assert_eq!(Cases::classified(&line), [format!("unrecognised {kind}"), "8 after 1".to_owned()]);
});

laws::law!(an_unknown_block_in_a_lead_record_is_unrecognised, "[a-z]{1,8}_unheard", block, {
    let fields =
        format!(r#""message":{{"model":"claude","content":[{{"type":"text","text":"said"}},{{"type":"{block}"}}]}}"#);
    prop_assert_eq!(Cases::assistant(&fields.as_str()), [format!("unrecognised {block}"), "8 after 1".to_owned()]);
});

laws::table!(
    every_surveyed_record_kind_without_a_voice_is_only_a_parent_edge,
    Cases::kind,
    [
        "ai-title" => ["8 after 1"],
        "api-request" => ["8 after 1"],
        "api-request-blob" => ["8 after 1"],
        "api-request-shape" => ["8 after 1"],
        "artifact-autoreact-ledger" => ["8 after 1"],
        "artifact-comment-monitor" => ["8 after 1"],
        "atis-latch" => ["8 after 1"],
        "cost-state" => ["8 after 1"],
        "file-history-delta" => ["8 after 1"],
        "file-history-snapshot" => ["8 after 1"],
        "frame-link" => ["8 after 1"],
        "last-prompt" => ["8 after 1"],
        "launched" => ["8 after 1"],
        "mode" => ["8 after 1"],
        "permission-mode" => ["8 after 1"],
        "pr-link" => ["8 after 1"],
        "queue-operation" => ["8 after 1"],
        "result" => ["8 after 1"],
        "started" => ["8 after 1"],
        "summary" => ["8 after 1"],
        "system" => ["8 after 1"],
        "tools-used" => ["8 after 1"],
        "brand-new-kind" => ["unrecognised brand-new-kind", "8 after 1"],
    ]
);

laws::table!(
    a_user_record_is_the_owner_only_when_a_human_typed_text_in_the_main_chain,
    Cases::user,
    [
        r#""origin":{"kind":"human"},"message":{"content":"typed"}"# => ["Owner typed"],
        r#""origin":{"kind":"human"},"message":{"content":[{"type":"image","source":{}},{"type":"text","text":"look"}]}"# => ["Owner look"],
        r#""origin":{"kind":"human"},"message":{"content":[{"type":"tool_result","content":"out"}]}"# => ["8 after 1"],
        r#""origin":{"kind":"human"},"isSidechain":true,"isMeta":true,"message":{"content":"relayed"}"# => ["8 after 1"],
        r#""origin":{"kind":"task-notification"},"message":{"content":"finished"}"# => ["8 after 1"],
        r#""origin":{"kind":"task-notification"},"isSidechain":true,"isMeta":true,"message":{"content":"finished"}"# => ["8 after 1"],
        r#""origin":{"kind":"coordinator"},"isSidechain":true,"isMeta":true,"message":{"content":"go"}"# => ["8 after 1"],
        r#""origin":{"kind":"peer"},"isMeta":true,"message":{"content":"hello"}"# => ["8 after 1"],
        r#""origin":{"kind":"peer"},"isSidechain":true,"isMeta":true,"message":{"content":"hello"}"# => ["8 after 1"],
        r#""origin":{"kind":"unclassified"},"isSidechain":true,"message":{"content":[{"type":"tool_result","content":"out"}]}"# => ["8 after 1"],
        r#""origin":{"kind":"unclassified"},"isSidechain":true,"message":{"content":"brief"}"# => ["8 after 1"],
        r#""origin":{"kind":"auto-continuation"},"isMeta":true,"message":{"content":"continue"}"# => ["8 after 1"],
        r#""origin":null,"message":{"content":[{"type":"tool_result","content":"out"}]}"# => ["8 after 1"],
        r#""isSidechain":true,"message":{"content":[{"type":"tool_result","content":"out"}]}"# => ["8 after 1"],
        r#""message":{"content":"<command-name>/compact</command-name>"}"# => ["8 after 1"],
        r#""isMeta":true,"message":{"content":"caveat"}"# => ["8 after 1"],
        r#""isSidechain":true,"message":{"content":"brief"}"# => ["8 after 1"],
        r#""isSidechain":true,"isMeta":true,"message":{"content":"brief"}"# => ["8 after 1"],
        r#""message":{"content":[{"type":"text","text":"[Request interrupted by user]"}]}"# => ["8 after 1"],
        r#""isMeta":true,"message":{"content":[{"type":"text","text":"skill"}]}"# => ["8 after 1"],
        r#""isSidechain":true,"message":{"content":[{"type":"text","text":"a"},{"type":"tool_result","content":"b"}]}"# => ["8 after 1"],
        r#""isCompactSummary":true,"isVisibleInTranscriptOnly":true,"message":{"content":"summary"}"# => ["8 after 1"],
        r#""origin":{"kind":"brand-new-kind"},"message":{"content":"hello"}"# => ["unrecognised brand-new-kind", "8 after 1"],
        r#""origin":{"kind":"brand-new-kind"},"message":{"content":[{"type":"tool_result","content":"out"}]}"# => ["unrecognised brand-new-kind", "8 after 1"],
        r#""origin":{"kind":"human"},"message":{"content":[{"type":"document"},{"type":"text","text":"a"}]}"# => ["unrecognised document", "8 after 1"],
    ]
);

laws::table!(
    an_assistant_record_is_the_lead_only_when_it_answered_with_text_in_the_main_chain,
    Cases::assistant,
    [
        r#""message":{"model":"claude","content":[{"type":"text","text":"said"}]}"# => ["Lead said"],
        r#""message":{"model":"claude","content":[{"type":"thinking","thinking":"hmm"}]}"# => ["8 after 1"],
        r#""message":{"model":"claude","content":[{"type":"tool_use","name":"Bash"}]}"# => ["8 after 1"],
        r#""isSidechain":true,"message":{"model":"claude","content":[{"type":"text","text":"subagent words"}]}"# => ["8 after 1"],
        r#""isSidechain":true,"message":{"model":"claude","content":[{"type":"thinking","thinking":"hmm"}]}"# => ["8 after 1"],
        r#""isSidechain":true,"message":{"model":"claude","content":[{"type":"tool_use","name":"Bash"}]}"# => ["8 after 1"],
        r#""isApiErrorMessage":true,"message":{"model":"<synthetic>","content":[{"type":"text","text":"API Error"}]}"# => ["8 after 1"],
        r#""isApiErrorMessage":true,"message":{"model":"claude","content":[{"type":"text","text":"API Error"}]}"# => ["8 after 1"],
        r#""isSidechain":true,"isApiErrorMessage":true,"message":{"model":"<synthetic>","content":[{"type":"text","text":"API Error"}]}"# => ["8 after 1"],
        r#""message":{"model":"<synthetic>","content":[{"type":"text","text":"No response requested."}]}"# => ["8 after 1"],
        r#""message":{"model":"claude","content":[{"type":"server_tool_use"}]}"# => ["unrecognised server_tool_use", "8 after 1"],
    ]
);

laws::table!(
    a_queued_command_is_the_owner_only_when_a_human_queued_it,
    Cases::attached,
    [
        r#"{"type":"queued_command","prompt":"typed while busy","origin":{"kind":"human"}}"# => ["Owner typed while busy"],
        r#"{"type":"queued_command","prompt":[{"type":"text","text":"see"},{"type":"image"}],"origin":{"kind":"human"}}"# => ["Owner see"],
        r#"{"type":"queued_command","prompt":"finished","origin":{"kind":"task-notification"}}"# => ["8 after 1"],
        r#"{"type":"queued_command","prompt":"go","origin":{"kind":"coordinator"}}"# => ["8 after 1"],
        r#"{"type":"queued_command","prompt":"hello","origin":{"kind":"peer"}}"# => ["8 after 1"],
        r#"{"type":"queued_command","prompt":"finished"}"# => ["8 after 1"],
        r#"{"type":"queued_command","prompt":"hello","origin":{"kind":"brand-new-kind"}}"# => ["unrecognised brand-new-kind", "8 after 1"],
        r#"{"type":"brand_new_attachment"}"# => ["unrecognised brand_new_attachment", "8 after 1"],
    ]
);

laws::table!(
    every_surveyed_attachment_other_than_a_queued_command_is_only_a_parent_edge,
    Cases::carried,
    [
        "agent_listing_delta" => ["8 after 1"],
        "auto_mode" => ["8 after 1"],
        "bash_output_audience_note" => ["8 after 1"],
        "batching_reminder_sent" => ["8 after 1"],
        "command_permissions" => ["8 after 1"],
        "compact_file_reference" => ["8 after 1"],
        "credential_org" => ["8 after 1"],
        "date" => ["8 after 1"],
        "deferred_tools_delta" => ["8 after 1"],
        "deferred_tools_record" => ["8 after 1"],
        "edited_text_file" => ["8 after 1"],
        "environment" => ["8 after 1"],
        "file" => ["8 after 1"],
        "hook_additional_context" => ["8 after 1"],
        "hook_blocking_error" => ["8 after 1"],
        "hook_non_blocking_error" => ["8 after 1"],
        "hook_permission_decision" => ["8 after 1"],
        "hook_success" => ["8 after 1"],
        "hook_system_message" => ["8 after 1"],
        "instructions" => ["8 after 1"],
        "invoked_skills" => ["8 after 1"],
        "mcp_instructions_delta" => ["8 after 1"],
        "model" => ["8 after 1"],
        "nested_memory" => ["8 after 1"],
        "prompt_snapshot" => ["8 after 1"],
        "read_truncation_notice" => ["8 after 1"],
        "remote_session_change" => ["8 after 1"],
        "session_context" => ["8 after 1"],
        "session_cron_carry" => ["8 after 1"],
        "silent_turn_reminder" => ["8 after 1"],
        "skill_listing" => ["8 after 1"],
        "task_reminder" => ["8 after 1"],
        "task_status" => ["8 after 1"],
        "thinking_drop" => ["8 after 1"],
        "total_tokens_reminder" => ["8 after 1"],
        "ultrathink_effort" => ["8 after 1"],
    ]
);

laws::roundtrip!(
    an_authorship_is_read_back_from_its_display,
    select(vec![Authorship::Owner, Authorship::Lead]),
    |author: &Authorship| author.to_string(),
    |name: &String| name.parse::<Authorship>().ok(),
    Some,
);

laws::table!(
    a_name_of_neither_party_is_no_authorship,
    |name: &&str| name.parse::<Authorship>().ok(),
    ["human" => None, "" => None],
);

laws::table!(
    an_instant_is_made_only_from_non_blank_text,
    |text: &&str| Instant::try_new((*text).to_owned()).ok().map(|instant| instant.to_string()),
    ["" => None, "2026-01-01T00:00:01Z" => Some(String::from("2026-01-01T00:00:01Z"))],
);

laws::table!(
    a_kind_is_made_only_from_non_blank_text,
    |text: &&str| Kind::try_new((*text).to_owned()).ok().map(|kind| kind.to_string()),
    ["" => None, "summary" => Some(String::from("summary"))],
);

laws::table!(
    a_session_id_is_made_only_from_non_blank_text,
    |text: &&str| SessionId::try_new((*text).to_owned()).ok().map(|session| session.to_string()),
    ["" => None, "s1" => Some(String::from("s1"))],
);

laws::roundtrip!(
    an_instant_is_rebuilt_from_its_display,
    any::<Instant>(),
    |instant: &Instant| instant.to_string(),
    |text: &String| Instant::try_new(text.clone()).ok(),
    Some,
);

laws::roundtrip!(
    a_kind_is_rebuilt_from_its_display,
    any::<Kind>(),
    |kind: &Kind| kind.to_string(),
    |text: &String| Kind::try_new(text.clone()).ok(),
    Some,
);

laws::roundtrip!(
    a_session_id_is_rebuilt_from_its_display,
    any::<SessionId>(),
    |session: &SessionId| session.to_string(),
    |text: &String| SessionId::try_new(text.clone()).ok(),
    Some,
);

laws::table!(
    a_body_holding_a_nul_is_refused_and_any_other_text_is_kept_whole,
    |text: &&str| transcript_ports::Body::try_new((*text).to_owned()).ok().map(|body| body.to_string()),
    [
        "said\0plainly" => None,
        "said plainly" => Some("said plainly".to_owned()),
    ]
);
