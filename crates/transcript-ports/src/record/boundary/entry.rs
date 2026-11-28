mod assistant;
mod attachment;
mod classified;
mod closing;
mod object;
mod said;
mod stamp;
mod user;
mod voice;

use invariants::Ignoring;
use serde_json::Value;

use crate::{Ignored, Line, Unrecognised};
use assistant::Assistant;
use attachment::Attached;
use classified::Classified;
pub use closing::Closing;
use object::Object;
use stamp::Passing;
use user::User;

#[derive(Debug)]
pub enum Known {
    User(User),
    Assistant(Assistant),
    Attachment(Attached),
    Passing(Passing),
}

#[derive(strum::EnumString, Clone, Copy, Debug)]
#[strum(serialize_all = "kebab-case")]
enum Tag {
    User,
    Assistant,
    Attachment,
    AiTitle,
    ApiRequest,
    ApiRequestBlob,
    ApiRequestShape,
    ArtifactAutoreactLedger,
    ArtifactCommentMonitor,
    AtisLatch,
    CostState,
    FileHistoryDelta,
    FileHistorySnapshot,
    FrameLink,
    LastPrompt,
    Launched,
    Mode,
    PermissionMode,
    PrLink,
    QueueOperation,
    Result,
    Started,
    Summary,
    System,
    ToolsUsed,
}

#[derive(Debug)]
pub enum Entry {
    Known(Known),
    Unknown(Unrecognised),
}

impl Entry {
    pub fn closing(&self) -> Closing {
        match self {
            Entry::Known(Known::Assistant(assistant)) => assistant.closing(),
            Entry::Known(Known::Attachment(attached)) => attached.closing(),
            Entry::Known(Known::User(_) | Known::Passing(_)) | Entry::Unknown(_) => Closing::Leaves,
        }
    }
}

impl Known {
    fn read(entry: Object<'_>) -> Option<Known> {
        let tag: Tag = entry.tag(Object::TYPE)?.parse().ignoring(Ignored::NotKnown)?;
        let read = match tag {
            Tag::User => User::read(entry).map(Known::User),
            Tag::Assistant => Assistant::read(entry).map(Known::Assistant),
            Tag::Attachment => Attached::read(entry).map(Known::Attachment),
            Tag::AiTitle
            | Tag::ApiRequest
            | Tag::ApiRequestBlob
            | Tag::ApiRequestShape
            | Tag::ArtifactAutoreactLedger
            | Tag::ArtifactCommentMonitor
            | Tag::AtisLatch
            | Tag::CostState
            | Tag::FileHistoryDelta
            | Tag::FileHistorySnapshot
            | Tag::FrameLink
            | Tag::LastPrompt
            | Tag::Launched
            | Tag::Mode
            | Tag::PermissionMode
            | Tag::PrLink
            | Tag::QueueOperation
            | Tag::Result
            | Tag::Started
            | Tag::Summary
            | Tag::System
            | Tag::ToolsUsed => entry.whole().map(Known::Passing),
        };
        read.ignoring(Ignored::NotKnown)
    }
}

impl Entry {
    pub fn read(value: &Value) -> Result<Entry, serde_json::Error> {
        let entry = Object::new(value);
        Known::read(entry).map_or_else(|| entry.whole().map(Entry::Unknown), |known| Ok(Entry::Known(known)))
    }
}

impl From<Known> for Option<Line> {
    fn from(known: Known) -> Option<Line> {
        match known {
            Known::User(user) => user.heard(),
            Known::Assistant(assistant) => assistant.heard(),
            Known::Attachment(attached) => attached.heard(),
            Known::Passing(passing) => passing.line(),
        }
    }
}

impl From<Entry> for Option<Line> {
    fn from(entry: Entry) -> Option<Line> {
        match entry {
            Entry::Known(known) => Option::<Line>::from(known),
            Entry::Unknown(unrecognised) => Some(Line::Unrecognised(unrecognised)),
        }
    }
}
