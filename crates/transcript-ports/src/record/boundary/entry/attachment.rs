use serde::Deserialize;

use super::classified::Classified;
use super::said::Said;
use super::stamp::Stamp;
use super::user::Wording;
use super::voice::{Listener, Origin};
use crate::{Line, Skipped, Unrecognised};

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Carried {
    QueuedCommand { prompt: Wording, origin: Option<Origin> },
    AgentListingDelta,
    AutoMode,
    BashOutputAudienceNote,
    BatchingReminderSent,
    CommandPermissions,
    CompactFileReference,
    CredentialOrg,
    Date,
    DeferredToolsDelta,
    DeferredToolsRecord,
    EditedTextFile,
    Environment,
    File,
    HookAdditionalContext,
    HookBlockingError,
    HookNonBlockingError,
    HookPermissionDecision,
    HookSuccess,
    HookSystemMessage,
    Instructions,
    InvokedSkills,
    McpInstructionsDelta,
    Model,
    NestedMemory,
    PromptSnapshot,
    ReadTruncationNotice,
    RemoteSessionChange,
    SessionContext,
    SessionCronCarry,
    SilentTurnReminder,
    SkillListing,
    TaskReminder,
    TaskStatus,
    ThinkingDrop,
    TotalTokensReminder,
    UltrathinkEffort,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Annex {
    Known(Carried),
    Unknown(Unrecognised),
}

#[derive(Debug, Deserialize)]
pub struct Attached {
    #[serde(flatten)]
    stamp: Stamp,
    attachment: Annex,
}

impl Carried {
    fn line(self, stamp: Stamp) -> Line {
        match self {
            Carried::QueuedCommand { prompt, origin } => Said::from(prompt).heard(Listener::new(stamp, origin)),
            _ => Line::Skipped(Skipped::from(stamp)),
        }
    }
}

impl Classified for Attached {
    fn line(self) -> Line {
        match self.attachment {
            Annex::Known(carried) => carried.line(self.stamp),
            Annex::Unknown(unrecognised) => self.stamp.unrecognised(unrecognised),
        }
    }
}
