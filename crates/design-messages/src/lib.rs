#![deny(clippy::wildcard_enum_match_arm)]

mod asked_earlier;
mod boundary;
mod cursors;
mod earlier_lines;
mod imported_notes;
mod prefixes;
mod quoted;
mod scope;
mod search_segments;
mod session_cursors;
mod session_records;
mod shown_message;
mod stored;
mod stream_schema;

pub use asked_earlier::{Asked, AskedEarlier};
pub use boundary::{Hit, LinkRow, RelationName, ShapeName};
pub use cursors::Cursors;
pub use earlier_lines::EarlierLines;
pub use imported_notes::{ImportedEnding, ImportedNotes, Noted};
pub use prefixes::{Prefixes, Shortest};
pub use quoted::{Quoted, QuotedSegments};
pub use scope::Scope;
pub use search_segments::SearchSegments;
pub use session_cursors::SessionCursors;
pub use session_records::{Holding, SessionRecords};
pub use shown_message::ShownMessage;
pub use stored::Stored;
pub use stream_schema::StreamSchema;
