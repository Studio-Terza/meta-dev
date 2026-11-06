use serde::Deserialize;

use crate::{Authorship, Body, Instant, Record, Uuid};

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
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Content {
    Text(String),
    Blocks(Vec<Block>),
    Other(serde::de::IgnoredAny),
}

#[derive(Debug, Deserialize)]
struct Message {
    content: Option<Content>,
}

#[derive(Debug, Deserialize)]
pub struct Entry {
    #[serde(default, rename = "type")]
    kind: Kind,
    uuid: Option<String>,
    #[serde(rename = "parentUuid")]
    parent: Option<String>,
    #[serde(rename = "sessionId")]
    session: Option<String>,
    timestamp: Option<String>,
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

impl Content {
    fn text(self) -> String {
        match self {
            Content::Text(text) => text,
            Content::Blocks(blocks) => blocks
                .into_iter()
                .filter(|block| matches!(block.kind, BlockKind::Text))
                .filter_map(|block| block.text)
                .collect::<Vec<_>>()
                .join("\n"),
            Content::Other(_) => String::new(),
        }
    }
}

impl From<Entry> for Option<Record> {
    fn from(entry: Entry) -> Option<Record> {
        let author = entry.kind.authorship(entry.origin)?;
        let text = entry.message.and_then(|message| message.content).map(Content::text).unwrap_or_default();
        Some(Record {
            uuid: Uuid::new(entry.uuid?),
            parent: entry.parent.map(Uuid::new),
            session: Uuid::new(entry.session?),
            at: Instant::new(entry.timestamp?),
            author,
            text: Body::new(text),
        })
    }
}
