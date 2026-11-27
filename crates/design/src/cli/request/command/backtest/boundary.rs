use grammar_lines::Lines;
use invariants::Ignoring;
use serde::Deserialize;
use transcript_ports::{Body, Uuid};

use crate::cli::request::ignored::Ignored;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum Entry {
    #[serde(rename = "user")]
    User { message: Message },
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

#[derive(Debug, Default)]
struct Reading {
    last: Option<Body>,
    logged: Vec<Logged>,
}

impl Logged {
    const INJECTED: &'static str = "Earlier owner words bearing on this:\n";

    pub fn read(log: &str) -> Vec<Logged> {
        let entries = log
            .parse::<Lines>()
            .into_iter()
            .flatten()
            .filter_map(|line| serde_json::from_str::<Entry>(&line).ignoring(Ignored::Unlogged));
        entries.fold(Reading::default(), Reading::heard).logged
    }

    pub fn said(&self) -> &Body {
        &self.said
    }

    pub fn quoted<'known>(&self, uuid: &'known Uuid) -> Option<&'known Uuid> {
        self.injected.contains(&format!("\n{uuid} ")).then_some(uuid)
    }
}

impl Reading {
    fn heard(self, entry: Entry) -> Reading {
        match entry {
            Entry::Assistant { message } => {
                Reading { last: message.content.written().or(self.last), logged: self.logged }
            }
            Entry::User { message: Message { content: Content::Said(text) } } if text.contains(Logged::INJECTED) => {
                let stop = self.last.clone().map(|said| Logged { said, injected: text });
                Reading { last: self.last, logged: self.logged.into_iter().chain(stop).collect() }
            }
            Entry::User { .. } | Entry::Other => self,
        }
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
