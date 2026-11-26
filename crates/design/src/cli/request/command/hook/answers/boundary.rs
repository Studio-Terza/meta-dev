use std::fmt;
use std::ops::Not;

use design_store::{Quote, Said};
use dispatch_ports::Reply;
use grammar_search::Terms;
use newtype::Wrapping;
use serde::Deserialize;
use transcript_ports::{Body, Uuid};

use crate::cli::request::ignored::Ignored;
use invariants::Ignoring;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Role(&'static str);

#[derive(Debug, Deserialize)]
pub struct Asked {
    kind: Kind,
    question: Body,
    #[serde(default)]
    terms: Vec<Body>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Kind {
    Question,
    Statement,
}

#[derive(Debug, Deserialize)]
struct Picked {
    uuid: Uuid,
    quote: Body,
}

#[derive(Debug, PartialEq)]
pub struct Answers(Vec<Quote>);

#[derive(Debug)]
pub struct Question {
    asked: Asked,
    candidates: Vec<Said>,
    answers: Option<Answers>,
}

#[derive(Debug)]
pub enum Traced {
    Statement,
    Question(Question),
}

#[derive(Debug)]
pub struct Classifying {
    role: Role,
    said: String,
}

#[derive(Debug)]
pub struct Selecting<'asked> {
    question: &'asked Body,
    candidates: &'asked [Said],
}

impl Traced {
    pub fn answers(self) -> Option<Answers> {
        match self {
            Traced::Statement => None,
            Traced::Question(question) => question.answers,
        }
    }
}

impl fmt::Display for Traced {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Traced::Statement => writeln!(formatter, "statement: asks the owner nothing"),
            Traced::Question(question) => writeln!(formatter, "{question}"),
        }
    }
}

impl Question {
    pub fn new(asked: Asked, candidates: Vec<Said>, answers: Option<Answers>) -> Question {
        Question { asked, candidates, answers }
    }
}

impl fmt::Display for Question {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let terms: Vec<String> = self.asked.terms.iter().map(ToString::to_string).collect();
        let found: Vec<String> = self.candidates.iter().map(|said| AsRef::<Uuid>::as_ref(said).to_string()).collect();
        writeln!(formatter, "question: {}", self.asked.question)?;
        writeln!(formatter, "terms: {}", terms.join(" "))?;
        writeln!(formatter, "candidates: {}", found.join(" "))?;
        let shown = self.answers.as_ref().map_or_else(|| String::from("answers: none"), ToString::to_string);
        write!(formatter, "{shown}")
    }
}

impl Role {
    pub const LEAD: Role = Role(
        "the lead agent of a software project, reporting to its owner; it asks when it wants the owner to answer a question, rule on something, approve or choose",
    );
    const KEPT: usize = 4000;

    pub fn classifying(self, said: &Body) -> Classifying {
        let text: &str = said.as_ref();
        let skipped = text.chars().count().saturating_sub(Self::KEPT);
        Classifying { role: self, said: text.chars().skip(skipped).collect() }
    }
}

impl Asked {
    pub fn read(reply: Reply) -> Option<Asked> {
        let json = Self::braced(&reply.into_inner(), '{', '}');
        serde_json::from_str::<Asked>(&json)
            .ignoring(Ignored::Unclassified)
            .filter(|asked| asked.kind == Kind::Question)
    }

    pub fn terms(&self) -> Option<Terms> {
        Terms::lexed(self.terms.iter().map(AsRef::as_ref))
    }

    pub fn selecting<'asked>(&'asked self, candidates: &'asked [Said]) -> Selecting<'asked> {
        Selecting { question: &self.question, candidates }
    }

    fn braced(text: &str, open: char, close: char) -> String {
        let letters: Vec<char> = text.chars().collect();
        let start = letters.iter().position(|letter| *letter == open).unwrap_or(letters.len());
        let end = letters.iter().rposition(|letter| *letter == close).map_or(start, |last| last + 1);
        letters.get(start..end).unwrap_or_default().iter().collect()
    }
}

impl Answers {
    pub fn verified(reply: &Reply, candidates: &[Said]) -> Option<Answers> {
        let json = Asked::braced(&reply.to_string(), '[', ']');
        let picked: Vec<Picked> = serde_json::from_str(&json).ignoring(Ignored::Unpicked).unwrap_or_default();
        let quotes: Vec<Quote> = picked
            .iter()
            .filter_map(|pick| candidates.iter().find_map(|said| said.quoted(&pick.uuid, &pick.quote)))
            .collect();
        quotes.is_empty().not().then_some(Answers(quotes))
    }
}

impl fmt::Display for Classifying {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "You read the last message an AI agent wrote at the end of its turn. The agent is {}.\n\
             Decide whether the message asks its owner a question, or asks the owner to make a decision, approve something, or choose between options.\n\
             Reply with JSON only, no prose and no code fence:\n\
             {{\"kind\": \"question\" or \"statement\", \"question\": the question restated in one line (empty for a statement), \"terms\": 6 to 14 single lowercase search words for finding where the owner may already have answered it: the question's key words, plus plain everyday synonyms and word forms the owner might have used instead, with compounds split into their words}}\n\n\
             Message:\n{}",
            self.role.0, self.said
        )
    }
}

impl fmt::Display for Selecting<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let candidates: Vec<String> = self.candidates.iter().map(ToString::to_string).collect();
        write!(
            formatter,
            "An AI agent is about to ask its owner this question:\n{}\n\n\
             Below are earlier messages from the owner, each headed by [uuid timestamp].\n\
             List every passage that answers the question or bears on it: a missed answer costs far more than an extra passage, so when unsure, include it.\n\
             Reply with JSON only, no prose and no code fence: a list of {{\"uuid\": the full uuid from the header, \"quote\": an excerpt of that message copied exactly, character for character, one or two sentences}}. Reply [] if none bear on it.\n\n{}",
            self.question,
            candidates.join("\n\n")
        )
    }
}

impl fmt::Display for Answers {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "The transcript may already answer this; reread before asking:")?;
        self.0.iter().try_for_each(|quote| writeln!(formatter, "{quote}"))
    }
}
