use serde::Deserialize;

use super::classified::Classified;
use super::said::{Hears, Said};
use super::stamp::Stamp;
use crate::{Authorship, Body, Line, Skipped, Unrecognised};

#[derive(Clone, Copy, Debug)]
struct Failed;

#[derive(Clone, Copy, Debug)]
struct Synthetic;

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(from = "bool")]
struct Delivery(Option<Failed>);

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(from = "String")]
struct Model(Option<Synthetic>);

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Written {
    Text { text: Body },
    Thinking,
    ToolUse,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Block {
    Known(Written),
    Unknown(Unrecognised),
}

#[derive(Debug, Deserialize)]
struct Message {
    model: Model,
    content: Vec<Block>,
}

#[derive(Debug, Deserialize)]
pub struct Assistant {
    #[serde(flatten)]
    stamp: Stamp,
    #[serde(rename = "isApiErrorMessage", default)]
    delivery: Delivery,
    message: Message,
}

#[derive(Debug)]
struct Answer {
    stamp: Stamp,
    delivery: Delivery,
    model: Model,
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
            Block::Known(Written::Thinking | Written::ToolUse) => Said::Silent,
            Block::Unknown(unrecognised) => Said::Unrecognised(unrecognised),
        }
    }
}

impl Hears for Answer {
    fn text(self, text: Body) -> Line {
        self.delivery.spoke(self.model, self.stamp, text)
    }

    fn silence(self) -> Line {
        Line::Skipped(Skipped::from(self.stamp))
    }

    fn unrecognised(self, unrecognised: Unrecognised) -> Line {
        self.stamp.unrecognised(unrecognised)
    }
}

impl Message {
    fn heard(self, stamp: Stamp, delivery: Delivery) -> Line {
        let said: Said = self.content.into_iter().map(Said::from).collect();
        said.heard(Answer { stamp, delivery, model: self.model })
    }
}

impl Classified for Assistant {
    fn line(self) -> Line {
        self.message.heard(self.stamp, self.delivery)
    }
}
