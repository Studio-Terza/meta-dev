mod assistant;
mod attachment;
mod classified;
mod said;
mod stamp;
mod user;
mod voice;

use serde::Deserialize;

use crate::{Line, Unrecognised};
use assistant::Assistant;
use attachment::Attached;
use classified::Classified;
use stamp::Passing;
use user::User;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Known {
    User(User),
    Assistant(Assistant),
    Attachment(Attached),
    AiTitle(Passing),
    ApiRequest(Passing),
    ApiRequestBlob(Passing),
    ApiRequestShape(Passing),
    ArtifactAutoreactLedger(Passing),
    ArtifactCommentMonitor(Passing),
    AtisLatch(Passing),
    CostState(Passing),
    FileHistoryDelta(Passing),
    FileHistorySnapshot(Passing),
    FrameLink(Passing),
    LastPrompt(Passing),
    Launched(Passing),
    Mode(Passing),
    PermissionMode(Passing),
    PrLink(Passing),
    QueueOperation(Passing),
    Result(Passing),
    Started(Passing),
    Summary(Passing),
    System(Passing),
    ToolsUsed(Passing),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Entry {
    Known(Known),
    Unknown(Unrecognised),
}

impl From<Known> for Option<Line> {
    fn from(known: Known) -> Option<Line> {
        match known {
            Known::User(user) => user.heard(),
            Known::Assistant(assistant) => assistant.heard(),
            Known::Attachment(attached) => attached.heard(),
            Known::AiTitle(passing)
            | Known::ApiRequest(passing)
            | Known::ApiRequestBlob(passing)
            | Known::ApiRequestShape(passing)
            | Known::ArtifactAutoreactLedger(passing)
            | Known::ArtifactCommentMonitor(passing)
            | Known::AtisLatch(passing)
            | Known::CostState(passing)
            | Known::FileHistoryDelta(passing)
            | Known::FileHistorySnapshot(passing)
            | Known::FrameLink(passing)
            | Known::LastPrompt(passing)
            | Known::Launched(passing)
            | Known::Mode(passing)
            | Known::PermissionMode(passing)
            | Known::PrLink(passing)
            | Known::QueueOperation(passing)
            | Known::Result(passing)
            | Known::Started(passing)
            | Known::Summary(passing)
            | Known::System(passing)
            | Known::ToolsUsed(passing) => passing.line(),
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
