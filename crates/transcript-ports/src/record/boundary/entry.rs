use serde::Deserialize;

use crate::{Authorship, Body, Instant, Record, SessionId, Uuid};

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Kind {
    User,
    Assistant,
    #[default]
    #[serde(other)]
    Other,
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
enum OriginKind {
    Human,
    #[default]
    #[serde(other)]
    Other,
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
enum BlockKind {
    Text,
    #[default]
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
struct Origin {
    #[serde(default)]
    kind: OriginKind,
}

#[derive(Debug, Deserialize)]
struct Block {
    #[serde(default, rename = "type")]
    kind: BlockKind,
    text: Option<Body>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Wording {
    Text(Body),
    Blocks(Vec<Block>),
    Other(serde::de::IgnoredAny),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Message {
    Said { content: Wording },
    Other(serde::de::IgnoredAny),
}

#[derive(Debug, Deserialize)]
pub struct Entry {
    #[serde(default, rename = "type")]
    kind: Kind,
    uuid: Option<Uuid>,
    #[serde(rename = "parentUuid")]
    parent: Option<Uuid>,
    #[serde(rename = "sessionId")]
    session: Option<SessionId>,
    timestamp: Option<Instant>,
    message: Option<Message>,
    origin: Option<Origin>,
}

impl OriginKind {
    fn authorship(self) -> Authorship {
        match self {
            OriginKind::Human => Authorship::Owner,
            OriginKind::Other => Authorship::Bystander,
        }
    }
}

impl Kind {
    fn authorship(self, origin: Option<Origin>) -> Option<Authorship> {
        match self {
            Kind::Assistant => Some(Authorship::Lead),
            Kind::User => Some(origin.map_or(Authorship::Bystander, |found| found.kind.authorship())),
            Kind::Other => None,
        }
    }
}

impl Block {
    fn written(self) -> Option<Body> {
        Some(self.kind).filter(|kind| matches!(kind, BlockKind::Text)).and(self.text)
    }
}

impl Wording {
    fn body(self) -> Body {
        match self {
            Wording::Text(text) => text,
            Wording::Blocks(blocks) => Body::new(
                blocks
                    .into_iter()
                    .filter_map(Block::written)
                    .map(|text| text.to_string())
                    .collect::<Vec<_>>()
                    .join(" "),
            ),
            Wording::Other(_) => Body::default(),
        }
    }
}

impl Message {
    fn body(self) -> Body {
        match self {
            Message::Said { content } => content.body(),
            Message::Other(_) => Body::default(),
        }
    }
}

impl Entry {
    pub fn record(self) -> Option<Record> {
        let author = self.kind.authorship(self.origin)?;
        let text = self.message.map_or_else(Body::default, Message::body);
        Some(Record {
            uuid: self.uuid?,
            parent: self.parent,
            session: self.session?,
            at: self.timestamp?,
            author,
            text,
        })
    }
}
