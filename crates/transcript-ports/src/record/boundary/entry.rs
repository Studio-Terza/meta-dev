use serde::Deserialize;

use crate::{Authorship, Body, Instant, Line, Record, SessionId, Skipped, Uuid};

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Kind {
    User,
    Assistant,
    Attachment,
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

#[derive(Debug, Default, Deserialize)]
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
#[serde(tag = "type", rename_all = "snake_case")]
enum Attached {
    QueuedCommand {
        prompt: Option<Wording>,
        origin: Option<Origin>,
    },
    #[serde(other)]
    Other,
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
    attachment: Option<Attached>,
}

struct Heard {
    origin: OriginKind,
    text: Option<Body>,
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

impl Attached {
    fn heard(self) -> Heard {
        match self {
            Attached::QueuedCommand { prompt, origin } => {
                Heard { origin: origin.unwrap_or_default().kind, text: prompt.and_then(Wording::body) }
            }
            Attached::Other => Heard { origin: OriginKind::Other, text: None },
        }
    }
}

impl Heard {
    fn author(&self, kind: Kind) -> Option<Authorship> {
        match kind {
            Kind::Assistant => Some(Authorship::Lead),
            Kind::User | Kind::Attachment => Some(self.origin.authorship()),
            Kind::Other => None,
        }
    }

    fn spoken(self, kind: Kind) -> Option<Spoken> {
        let author = self.author(kind)?;
        self.text.map(|text| Spoken { author, text })
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
        let spoken = self.voice().spoken(self.kind);
        self.heard(spoken)
    }

    fn voice(&mut self) -> Heard {
        match self.attachment.take() {
            Some(attached) => attached.heard(),
            None => Heard {
                origin: self.origin.take().unwrap_or_default().kind,
                text: self.message.take().and_then(Message::body),
            },
        }
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
