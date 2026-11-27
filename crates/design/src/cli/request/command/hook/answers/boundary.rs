use std::collections::BTreeSet;
use std::fmt;
use std::ops::Not;

use design_messages::{Quote, Said, StopRow, Tally, Unabridged};
use dispatch_ports::{Prompt, Reply};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use transcript_ports::{Body, Uuid};

use super::super::super::crew::Heard;
use crate::cli::request::ignored::Ignored;
use invariants::Ignoring;

#[derive(Debug)]
pub(crate) struct Asked {
    question: Body,
}

#[derive(Debug)]
pub(crate) struct Reading<'history> {
    chunks: Vec<&'history [Said]>,
    prompts: Vec<Prompt>,
}

#[derive(Debug)]
pub(crate) struct Trimming<'asked> {
    question: &'asked Body,
    quotes: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Picked {
    uuid: Uuid,
    quote: Body,
}

#[derive(Debug, PartialEq)]
pub struct Answers(Vec<Quote>);

#[derive(Debug)]
pub struct Search {
    read: Tally,
    quoted: Tally,
    answers: Option<Answers>,
    tokens: Tally,
}

#[derive(Debug)]
pub(crate) struct Selecting<'asked> {
    question: &'asked Body,
    shown: Vec<String>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Replied<'reply>(&'reply Reply);

impl Search {
    pub fn new(history: &[Said], quoted: &[Quote], answers: Option<Answers>, tokens: Tally) -> Search {
        Search { read: Tally::from(history.len()), quoted: Tally::from(quoted.len()), answers, tokens }
    }

    pub fn answers(&self) -> Option<&Answers> {
        self.answers.as_ref()
    }

    pub(crate) fn trailed(&self, row: StopRow) -> StopRow {
        let kept = Tally::from(self.answers.as_ref().map_or(0, |answers| answers.0.len()));
        row.searched([self.read, self.quoted, kept, self.tokens])
    }
}

impl From<&Body> for Asked {
    fn from(said: &Body) -> Asked {
        Asked { question: said.clone() }
    }
}

impl Asked {
    fn selecting(&self, shown: &[impl fmt::Display]) -> Selecting<'_> {
        Selecting { question: &self.question, shown: shown.iter().map(ToString::to_string).collect() }
    }

    pub(crate) fn reading<'history>(&self, history: &'history [Said]) -> Reading<'history> {
        let chunks: Vec<&[Said]> = history.chunks(Reading::CHUNK).collect();
        let prompts = chunks
            .iter()
            .map(|chunk| {
                let whole: Vec<Unabridged<'_>> = chunk.iter().map(Unabridged::from).collect();
                Prompt::from(&self.selecting(&whole))
            })
            .collect();
        Reading { chunks, prompts }
    }

    pub(crate) fn trimming(&self, quotes: &[Quote]) -> Trimming<'_> {
        Trimming {
            question: &self.question,
            quotes: quotes.iter().zip(1..).map(|(quote, number)| format!("{number}. {quote}")).collect(),
        }
    }
}

impl Reading<'_> {
    const CHUNK: usize = 300;

    pub(crate) fn prompts(&self) -> &[Prompt] {
        &self.prompts
    }

    pub fn quoted(&self, heard: &[Heard]) -> Vec<Vec<Quote>> {
        self.chunks
            .iter()
            .zip(heard)
            .map(|(chunk, read)| read.reply().map_or_default(|reply| Answers::quoted(reply, chunk)))
            .collect()
    }
}

impl Replied<'_> {
    pub(crate) fn listed<T: DeserializeOwned>(self) -> Vec<T> {
        serde_json::from_str(&self.braced('[', ']')).ignoring(Ignored::Unpicked).unwrap_or_default()
    }

    fn braced(self, open: char, close: char) -> String {
        let letters: Vec<char> = self.0.to_string().chars().collect();
        let start = letters.iter().position(|letter| *letter == open).unwrap_or(letters.len());
        let end = letters.iter().rposition(|letter| *letter == close).map_or(start, |last| last + 1);
        letters.get(start..end).unwrap_or_default().iter().collect()
    }
}

impl Answers {
    const KEPT: usize = 8;

    pub(crate) fn kept(reply: &Reply, quotes: &[Quote]) -> Option<Answers> {
        let named: BTreeSet<usize> = Replied(reply).listed::<usize>().into_iter().collect();
        let kept: Vec<Quote> = quotes
            .iter()
            .zip(1..)
            .filter(|(_quote, number)| named.contains(number))
            .map(|(quote, _number)| quote.clone())
            .take(Answers::KEPT)
            .collect();
        kept.is_empty().not().then_some(Answers(kept))
    }

    pub fn quoted(reply: &Reply, candidates: &[Said]) -> Vec<Quote> {
        let picked: Vec<Picked> = Replied(reply).listed();
        picked
            .iter()
            .filter_map(|pick| candidates.iter().find_map(|said| said.quoted(&pick.uuid, &pick.quote)))
            .collect()
    }
}

impl fmt::Display for Selecting<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "An AI agent just ended its turn with this message to its owner:\n{}\n\n\
             Below are earlier messages from the owner, each headed by [uuid timestamp].\n\
             List every passage that bears on the message: an earlier answer to anything it asks, a decision, rule, preference or instruction it should follow, or the next step the owner set. A missed passage costs far more than an extra one, so when unsure, include it.\n\
             Reply with JSON only, no prose and no code fence: a list of {{\"uuid\": the full uuid from the header, \"quote\": an excerpt of that message copied exactly, character for character, one or two sentences}}. Reply [] if none bear on it.\n\n{}",
            self.question,
            self.shown.join("\n\n")
        )
    }
}

impl fmt::Display for Trimming<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "An AI agent just ended its turn with this message to its owner:\n{}\n\n\
             Below, numbered, are passages the owner wrote earlier, each with its message uuid and timestamp.\n\
             Keep only the passages the agent needs now: an earlier answer to what it asks, or a decision, rule, preference or next step that bears directly on what it says it will do. Keep at most {}.\n\
             Reply with JSON only, no prose and no code fence: a list of passage numbers, such as [3, 1]. Reply [] if none bear on it.\n\n{}",
            self.question,
            Answers::KEPT,
            self.quotes.join("\n")
        )
    }
}

impl fmt::Display for Answers {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "Earlier owner words bearing on this:")?;
        self.0.iter().try_for_each(|quote| writeln!(formatter, "{quote}"))
    }
}
