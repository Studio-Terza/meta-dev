use invariants::Ignoring;
use json_ports::{JsonFailure, Object};

use super::classified::Classified;
use super::closing::Closing;
use super::said::Said;
use super::stamp::Stamp;
use super::user::Wording;
use super::voice::{Listener, Origin};
use crate::{Ignored, Line, Skipped, Unrecognised};

#[derive(strum::EnumString, Debug)]
#[strum(serialize_all = "snake_case")]
enum Carried {
    #[strum(disabled)]
    QueuedCommand {
        prompt: Wording,
        origin: Option<Origin>,
    },
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

#[derive(Debug)]
enum Annex {
    Known(Carried),
    Unknown(Unrecognised),
}

#[derive(Clone, Copy, Debug, serde::Deserialize, PartialEq, Eq)]
enum Event {
    SubagentStop,
    #[serde(other)]
    Other,
}

#[derive(Debug)]
pub struct Attached {
    stamp: Stamp,
    attachment: Annex,
    event: Option<Event>,
}

impl Attached {
    const ATTACHMENT: &'static str = "attachment";
    const EVENT: &'static str = "hookEvent";

    pub fn read(entry: Object<'_>) -> Result<Attached, JsonFailure> {
        let attachment = entry.required(Self::ATTACHMENT)?;
        let event = attachment.field(Self::EVENT).ignoring(Ignored::NotKnown).flatten();
        Ok(Attached { stamp: entry.whole()?, attachment: Annex::read(attachment)?, event })
    }

    pub fn closing(&self) -> Closing {
        match (&self.attachment, self.event) {
            (Annex::Known(Carried::PromptSnapshot), _)
            | (Annex::Known(Carried::HookSuccess | Carried::HookNonBlockingError), Some(Event::SubagentStop)) => {
                Closing::Trails
            }
            (Annex::Known(_) | Annex::Unknown(_), _) => Closing::Leaves,
        }
    }
}

impl Annex {
    fn read(attachment: Object<'_>) -> Result<Annex, JsonFailure> {
        Carried::read(attachment)
            .map_or_else(|| attachment.whole().map(Annex::Unknown), |carried| Ok(Annex::Known(carried)))
    }
}

impl Carried {
    const QUEUED: &'static str = "queued_command";
    const PROMPT: &'static str = "prompt";
    const ORIGIN: &'static str = "origin";

    fn read(attachment: Object<'_>) -> Option<Carried> {
        match attachment.tag(Object::TYPE)? {
            Carried::QUEUED => Carried::queued(attachment).ignoring(Ignored::NotKnown),
            other => other.parse().ignoring(Ignored::NotKnown),
        }
    }

    fn queued(attachment: Object<'_>) -> Result<Carried, JsonFailure> {
        let origin = attachment.present(Self::ORIGIN).map(Origin::read).transpose()?;
        Ok(Carried::QueuedCommand { prompt: Wording::read(attachment.required(Self::PROMPT)?)?, origin })
    }

    fn line(self, stamp: Stamp) -> Line {
        match self {
            Carried::QueuedCommand { prompt, origin } => Said::from(prompt).heard(Listener::new(stamp, origin)),
            Carried::AgentListingDelta
            | Carried::AutoMode
            | Carried::BashOutputAudienceNote
            | Carried::BatchingReminderSent
            | Carried::CommandPermissions
            | Carried::CompactFileReference
            | Carried::CredentialOrg
            | Carried::Date
            | Carried::DeferredToolsDelta
            | Carried::DeferredToolsRecord
            | Carried::EditedTextFile
            | Carried::Environment
            | Carried::File
            | Carried::HookAdditionalContext
            | Carried::HookBlockingError
            | Carried::HookNonBlockingError
            | Carried::HookPermissionDecision
            | Carried::HookSuccess
            | Carried::HookSystemMessage
            | Carried::Instructions
            | Carried::InvokedSkills
            | Carried::McpInstructionsDelta
            | Carried::Model
            | Carried::NestedMemory
            | Carried::PromptSnapshot
            | Carried::ReadTruncationNotice
            | Carried::RemoteSessionChange
            | Carried::SessionContext
            | Carried::SessionCronCarry
            | Carried::SilentTurnReminder
            | Carried::SkillListing
            | Carried::TaskReminder
            | Carried::TaskStatus
            | Carried::ThinkingDrop
            | Carried::TotalTokensReminder
            | Carried::UltrathinkEffort => Line::Skipped(Skipped::from(stamp)),
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
