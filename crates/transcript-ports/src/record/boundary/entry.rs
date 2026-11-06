use serde::Deserialize;

use crate::{Authorship, Body, Instant, Line, Record, SessionId, Skipped, Uuid};

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

struct Spoken {
    author: Authorship,
    text: Body,
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
    fn spoken(self, origin: Option<&Origin>, text: Option<Body>) -> Option<Spoken> {
        match self {
            Kind::Assistant => text.map(|text| Spoken { author: Authorship::Lead, text }),
            Kind::User => Some(Spoken {
                author: origin.map_or(Authorship::Bystander, |found| found.kind.authorship()),
                text: text.unwrap_or_default(),
            }),
            Kind::Other => None,
        }
    }
}

impl Block {
    fn written(self) -> Option<Body> {
        match self.kind {
            BlockKind::Text => self.text,
            BlockKind::Other => None,
        }
    }
}

impl Wording {
    fn body(self) -> Option<Body> {
        match self {
            Wording::Text(text) => Some(text),
            Wording::Blocks(blocks) => blocks
                .into_iter()
                .filter_map(Block::written)
                .reduce(|joined, text| Body::new(format!("{joined} {text}"))),
            Wording::Other(_) => None,
        }
    }
}

impl Message {
    fn body(self) -> Option<Body> {
        match self {
            Message::Said { content } => content.body(),
            Message::Other(_) => None,
        }
    }
}

impl Entry {
    pub fn line(mut self) -> Option<Line> {
        let text = self.message.take().and_then(Message::body);
        let spoken = self.kind.spoken(self.origin.as_ref(), text);
        self.heard(spoken)
    }

    fn heard(self, spoken: Option<Spoken>) -> Option<Line> {
        match spoken {
            Some(spoken) => self.said(spoken).map(Line::Said),
            None => self.skipped().map(Line::Skipped),
        }
    }

    fn said(self, spoken: Spoken) -> Option<Record> {
        Some(Record {
            uuid: self.uuid?,
            parent: self.parent,
            session: self.session?,
            at: self.timestamp?,
            author: spoken.author,
            text: spoken.text,
        })
    }

    fn skipped(self) -> Option<Skipped> {
        self.uuid.map(|uuid| Skipped::new(uuid, self.parent))
    }
}
