use invariants::Ignoring;
use json_ports::{JsonFailure, Object};

use super::classified::Classified;
use super::closing::Closing;
use super::said::{Hears, Said};
use super::stamp::Stamp;
use crate::{Authorship, Body, Ignored, Line, Skipped, TaskId, Unrecognised};

#[derive(Clone, Copy, Debug)]
struct Failed;

#[derive(Clone, Copy, Debug)]
struct Synthetic;

#[derive(Clone, Copy, Debug, Default, serde::Deserialize)]
#[serde(from = "bool")]
struct Delivery(Option<Failed>);

#[derive(Clone, Copy, Debug, serde::Deserialize)]
#[serde(from = "String")]
struct Model(Option<Synthetic>);

#[derive(Debug)]
enum Written {
    Text { text: Body },
    Thinking,
    ToolUse { stopped: Option<TaskId> },
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "name")]
enum Used {
    TaskStop {
        input: Stopping,
    },
    #[serde(other)]
    Other,
}

#[derive(Debug, serde::Deserialize)]
struct Stopping {
    task_id: TaskId,
}

#[derive(strum::EnumString, Clone, Copy, Debug)]
#[strum(serialize_all = "snake_case")]
enum Tag {
    Text,
    Thinking,
    ToolUse,
}

#[derive(Debug)]
enum Block {
    Known(Written),
    Unknown(Unrecognised),
}

#[derive(Clone, Copy, Debug, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Reason {
    EndTurn,
    StopSequence,
    #[serde(other)]
    Other,
}

#[derive(Debug)]
struct Message {
    model: Model,
    content: Vec<Block>,
    reason: Option<Reason>,
}

#[derive(Debug)]
pub struct Assistant {
    stamp: Stamp,
    delivery: Delivery,
    message: Message,
}

#[derive(Debug)]
struct Answer {
    stamp: Stamp,
    delivery: Delivery,
    model: Model,
    stopped: Option<TaskId>,
}

impl Assistant {
    const DELIVERY: &'static str = "isApiErrorMessage";
    const MESSAGE: &'static str = "message";

    pub fn read(entry: Object<'_>) -> Result<Assistant, JsonFailure> {
        let delivery = entry.present(Self::DELIVERY).map_or_else(|| Ok(Delivery::default()), Object::whole)?;
        Ok(Assistant { stamp: entry.whole()?, delivery, message: Message::read(entry.required(Self::MESSAGE)?)? })
    }

    pub fn closing(&self) -> Closing {
        match self.message.reason {
            Some(Reason::EndTurn | Reason::StopSequence) => Closing::Ends,
            Some(Reason::Other) | None => Closing::Leaves,
        }
    }
}

impl Message {
    const MODEL: &'static str = "model";
    const CONTENT: &'static str = "content";
    const REASON: &'static str = "stop_reason";

    fn read(message: Object<'_>) -> Result<Message, JsonFailure> {
        let content: Vec<Block> =
            message.required(Self::CONTENT)?.each()?.into_iter().map(Block::read).collect::<Result<_, _>>()?;
        let reason = message.field(Self::REASON).ignoring(Ignored::NotKnown).flatten();
        Ok(Message { model: message.field(Self::MODEL)?, content, reason })
    }
}

impl Block {
    fn read(block: Object<'_>) -> Result<Block, JsonFailure> {
        Written::read(block).map_or_else(|| block.whole().map(Block::Unknown), |written| Ok(Block::Known(written)))
    }
}

impl Written {
    const TEXT: &'static str = "text";

    fn read(block: Object<'_>) -> Option<Written> {
        let tag: Tag = block.tag(Object::TYPE)?.parse().ignoring(Ignored::NotKnown)?;
        match tag {
            Tag::Text => block.field(Self::TEXT).ignoring(Ignored::NotKnown).map(|text| Written::Text { text }),
            Tag::Thinking => Some(Written::Thinking),
            Tag::ToolUse => {
                Some(Written::ToolUse { stopped: block.whole().ignoring(Ignored::Unstopped).and_then(Used::stopped) })
            }
        }
    }
}

impl Used {
    fn stopped(self) -> Option<TaskId> {
        match self {
            Used::TaskStop { input } => Some(input.task_id),
            Used::Other => None,
        }
    }
}

impl Block {
    fn stopped(&self) -> Option<TaskId> {
        match self {
            Block::Known(Written::ToolUse { stopped }) => stopped.clone(),
            Block::Known(Written::Text { .. } | Written::Thinking) | Block::Unknown(_) => None,
        }
    }
}

impl From<bool> for Delivery {
    fn from(failed: bool) -> Delivery {
        Delivery(failed.then_some(Failed))
    }
}

impl From<String> for Model {
    fn from(name: String) -> Model {
        Model((name == Model::SYNTHETIC).then_some(Synthetic))
    }
}

impl Model {
    const SYNTHETIC: &str = "<synthetic>";

    fn spoke(self, stamp: Stamp, text: Body) -> Line {
        match self.0 {
            None => stamp.said(Authorship::Lead, text),
            Some(Synthetic) => Line::Skipped(Skipped::from(stamp)),
        }
    }
}

impl Delivery {
    fn spoke(self, model: Model, stamp: Stamp, text: Body) -> Line {
        match self.0 {
            None => model.spoke(stamp, text),
            Some(Failed) => Line::Skipped(Skipped::from(stamp)),
        }
    }
}

impl From<Block> for Said {
    fn from(block: Block) -> Said {
        match block {
            Block::Known(Written::Text { text }) => Said::Text(text),
            Block::Known(Written::Thinking | Written::ToolUse { .. }) => Said::Silent,
            Block::Unknown(unrecognised) => Said::Unrecognised(unrecognised),
        }
    }
}

impl Hears for Answer {
    fn text(self, text: Body) -> Line {
        self.delivery.spoke(self.model, self.stamp, text)
    }

    fn silence(self) -> Line {
        let stamp = self.stamp;
        self.stopped.map_or_else(|| Line::Skipped(Skipped::from(stamp)), Line::Ended)
    }

    fn unrecognised(self, unrecognised: Unrecognised) -> Line {
        self.stamp.unrecognised(unrecognised)
    }
}

impl Message {
    fn heard(self, stamp: Stamp, delivery: Delivery) -> Line {
        let stopped = self.content.iter().find_map(Block::stopped);
        let said: Said = self.content.into_iter().map(Said::from).collect();
        said.heard(Answer { stamp, delivery, model: self.model, stopped })
    }
}

impl Classified for Assistant {
    fn line(self) -> Line {
        self.message.heard(self.stamp, self.delivery)
    }
}
