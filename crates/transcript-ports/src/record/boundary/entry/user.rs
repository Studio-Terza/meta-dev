use invariants::Ignoring;
use json_ports::{JsonFailure, Object};

use super::classified::Classified;
use super::said::Said;
use super::stamp::Stamp;
use super::voice::{Listener, Origin};
use crate::{Body, Ignored, Line, Unrecognised};

#[derive(Debug)]
pub enum Heard {
    Text { text: Body },
    ToolResult,
    Image,
}

#[derive(strum::EnumString, Clone, Copy, Debug)]
#[strum(serialize_all = "snake_case")]
enum Tag {
    Text,
    ToolResult,
    Image,
}

#[derive(Debug)]
pub enum Block {
    Known(Heard),
    Unknown(Unrecognised),
}

#[derive(Debug)]
pub enum Wording {
    Typed(Body),
    Blocks(Vec<Block>),
}

#[derive(Debug)]
pub struct User {
    stamp: Stamp,
    origin: Option<Origin>,
    content: Wording,
}

impl User {
    const ORIGIN: &'static str = "origin";
    const MESSAGE: &'static str = "message";
    const CONTENT: &'static str = "content";

    pub fn read(entry: Object<'_>) -> Result<User, JsonFailure> {
        let origin = entry.present(Self::ORIGIN).map(Origin::read).transpose()?;
        let content = Wording::read(entry.required(Self::MESSAGE)?.required(Self::CONTENT)?)?;
        Ok(User { stamp: entry.whole()?, origin, content })
    }
}

impl Wording {
    pub fn read(content: Object<'_>) -> Result<Wording, JsonFailure> {
        content.whole().ignoring(Ignored::NotKnown).map_or_else(
            || content.each()?.into_iter().map(Block::read).collect::<Result<_, _>>().map(Wording::Blocks),
            |typed| Ok(Wording::Typed(typed)),
        )
    }
}

impl Block {
    fn read(block: Object<'_>) -> Result<Block, JsonFailure> {
        Heard::read(block).map_or_else(|| block.whole().map(Block::Unknown), |heard| Ok(Block::Known(heard)))
    }
}

impl Heard {
    const TEXT: &'static str = "text";

    fn read(block: Object<'_>) -> Option<Heard> {
        let tag: Tag = block.tag(Object::TYPE)?.parse().ignoring(Ignored::NotKnown)?;
        match tag {
            Tag::Text => block.field(Self::TEXT).ignoring(Ignored::NotKnown).map(|text| Heard::Text { text }),
            Tag::ToolResult => Some(Heard::ToolResult),
            Tag::Image => Some(Heard::Image),
        }
    }
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
        Said::from(self.content).heard(Listener::new(self.stamp, self.origin))
    }
}
