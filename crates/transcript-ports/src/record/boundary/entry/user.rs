use serde::Deserialize;

use super::classified::Classified;
use super::said::Said;
use super::stamp::Stamp;
use super::voice::{Listener, Origin};
use crate::{Body, Line, Unrecognised};

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Heard {
    Text { text: Body },
    ToolResult,
    Image,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Block {
    Known(Heard),
    Unknown(Unrecognised),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Wording {
    Typed(Body),
    Blocks(Vec<Block>),
}

#[derive(Debug, Deserialize)]
struct Utterance {
    content: Wording,
}

#[derive(Debug, Deserialize)]
pub struct User {
    #[serde(flatten)]
    stamp: Stamp,
    origin: Option<Origin>,
    message: Utterance,
}

impl From<Block> for Said {
    fn from(block: Block) -> Said {
        match block {
            Block::Known(Heard::Text { text }) => Said::Text(text),
            Block::Known(Heard::ToolResult | Heard::Image) => Said::Silent,
            Block::Unknown(unrecognised) => Said::Unrecognised(unrecognised),
        }
    }
}

impl From<Wording> for Said {
    fn from(content: Wording) -> Said {
        match content {
            Wording::Typed(text) => Said::Text(text),
            Wording::Blocks(blocks) => blocks.into_iter().map(Said::from).collect(),
        }
    }
}

impl Classified for User {
    fn line(self) -> Line {
        Said::from(self.message.content).heard(Listener::new(self.stamp, self.origin))
    }
}
