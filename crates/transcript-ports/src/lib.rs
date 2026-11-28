#![deny(clippy::wildcard_enum_match_arm)]
mod authorship;
mod blank;
mod body;
mod claude_code_dir;
mod cursor;
mod instant;
mod kind;
mod line;
mod lossy_text;
mod read;
mod read_failure;
mod record;
mod session;
mod session_id;
mod skipped;
mod task_id;
mod transcripts;
mod turn;
mod unrecognised;
mod uuid;

pub use authorship::Authorship;
pub use body::Body;
pub use claude_code_dir::ClaudeCodeDir;
pub use cursor::Cursor;
pub use instant::Instant;
pub use kind::Kind;
pub use line::Line;
pub use read::Read;
pub use read_failure::ReadFailure;
pub use record::Record;
pub use session::Session;
pub use session_id::SessionId;
pub use skipped::Skipped;
pub use task_id::TaskId;
pub use transcripts::{Transcripts, transcripts_ops};
pub use turn::Turn;
pub use unrecognised::Unrecognised;
pub use uuid::Uuid;

invariants::ignored! {
    pub(crate) enum Ignored {
        Unparsed => "a line that does not parse as an entry is kept as an unreadable unrecognised line",
        NotKnown => "a body whose tag is not a known kind, or that does not read as its kind, is read as unrecognised",
        Unstopped => "a tool use that does not read as a TaskStop naming a task stops no task",
        Oversized => "text pieces whose join passes u32::MAX bytes say nothing",
    }
}
