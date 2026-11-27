use grammar_lines::Lines;
use invariants::Ignoring;
use serde::Deserialize;
use transcript_ports::{Body, Uuid};

use crate::cli::request::ignored::Ignored;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum Entry {
    #[serde(rename = "user")]
    User { message: Message, uuid: Option<Uuid> },
    #[serde(rename = "assistant")]
    Assistant { message: Message },
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: Content,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Content {
    Said(String),
    Blocks(Vec<Block>),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum Block {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(other)]
    Other,
}

#[derive(Debug)]
pub struct Logged {
    said: Body,
    injected: String,
}

#[derive(Debug)]
pub enum Spoken {
    Lead(Body),
    Owner { uuid: Uuid, said: Body },
}

#[derive(Debug, Default)]
pub struct Log {
    last: Option<Body>,
    stops: Vec<Logged>,
    spoken: Vec<Spoken>,
}

impl Log {
    pub fn read(log: &str) -> Log {
        let entries = log
            .parse::<Lines>()
            .into_iter()
            .flatten()
            .filter_map(|line| serde_json::from_str::<Entry>(&line).ignoring(Ignored::Unlogged));
        entries.fold(Log::default(), Log::heard)
    }

    pub fn stops(&self) -> &[Logged] {
        &self.stops
    }

    pub fn spoken(&self) -> &[Spoken] {
        &self.spoken
    }

    fn heard(mut self, entry: Entry) -> Log {
        match entry {
            Entry::Assistant { message } => {
                let written = message.content.written();
                self.spoken.extend(written.iter().cloned().map(Spoken::Lead));
                Log { last: written.or(self.last), ..self }
            }
            Entry::User { message: Message { content: Content::Said(text) }, uuid } => self.said(text, uuid),
            Entry::User { .. } | Entry::Other => self,
        }
    }

    fn said(self, text: String, uuid: Option<Uuid>) -> Log {
        if text.contains(Logged::INJECTED) { self.injected(text) } else { self.owned(text, uuid) }
    }

    fn injected(mut self, text: String) -> Log {
        self.stops.extend(self.last.clone().map(|said| Logged { said, injected: text }));
        self
    }

    fn owned(mut self, text: String, uuid: Option<Uuid>) -> Log {
        let owner = uuid.zip(Body::try_new(text).ignoring(Ignored::Unlogged));
        self.spoken.extend(owner.map(|(owned, said)| Spoken::Owner { uuid: owned, said }));
        self
    }
}

impl Logged {
    const INJECTED: &'static str = "Earlier owner words bearing on this:\n";

    pub fn said(&self) -> &Body {
        &self.said
    }

    pub fn quoted<'known>(&self, uuid: &'known Uuid) -> Option<&'known Uuid> {
        self.injected.contains(&format!("\n{uuid} ")).then_some(uuid)
    }
}

impl Content {
    fn written(self) -> Option<Body> {
        let texts: Vec<String> = match self {
            Content::Said(text) => vec![text],
            Content::Blocks(blocks) => blocks.into_iter().filter_map(Block::text).collect(),
        };
        texts.first().and_then(|_| Body::try_new(texts.join("\n")).ignoring(Ignored::Unlogged))
    }
}

impl Block {
    fn text(self) -> Option<String> {
        match self {
            Block::Text { text } => Some(text),
            Block::Other => None,
        }
    }
}
