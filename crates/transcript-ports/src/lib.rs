#![deny(clippy::wildcard_enum_match_arm)]
mod blank;
mod boundary;
mod claude_code_dir;
mod cursor;
mod line;
mod read;
mod read_failure;
mod record;
mod session;
mod skipped;
mod task_id;
mod transcripts;
mod unrecognised;

pub use boundary::{Authorship, Body, Instant, Kind, SessionId, Uuid};
pub use claude_code_dir::ClaudeCodeDir;
pub use cursor::Cursor;
pub use line::Line;
pub use read::Read;
pub use read_failure::ReadFailure;
pub use record::Record;
pub use session::Session;
pub use skipped::Skipped;
pub use task_id::TaskId;
pub use transcripts::{Transcripts, transcripts_ops};
use unrecognised::Unrecognised;

invariants::ignored! {
    pub(crate) enum Ignored {
        Unparsed => "a line that does not parse as an entry is kept as an unreadable unrecognised line",
        NotKnown => "a body whose tag is not a known kind, or that does not read as its kind, is read as unrecognised",
        Unstopped => "a tool use that does not read as a TaskStop naming a task stops no task",
        Oversized => "text pieces whose join passes u32::MAX bytes say nothing",
    }
}
