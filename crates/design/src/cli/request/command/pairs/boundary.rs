use std::collections::BTreeSet;
use std::fmt;

use grammar_lines::Lines;
use invariants::Ignoring;
use serde::Deserialize;
use transcript_ports::{Body, Uuid};

use super::super::backtest::Spoken;
use super::super::crew::Heard;
use super::super::hook::Replied;
use crate::cli::request::ignored::Ignored;

#[derive(Clone, Copy, Debug)]
pub struct Turn<'log> {
    at: usize,
    spoken: &'log Spoken,
}

#[derive(Debug)]
pub struct Excerpt<'log> {
    turns: Vec<Turn<'log>>,
    chars: usize,
}

#[derive(Debug, Deserialize)]
struct Picked {
    question: Body,
    answer_uuid: Uuid,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pair {
    at: usize,
    question: Body,
    answer_uuid: Uuid,
}

#[derive(Debug)]
pub struct PairLines<'pairs>(&'pairs [Pair]);

impl<'log> Turn<'log> {
    pub fn kept(at: usize, spoken: &'log Spoken, known: &BTreeSet<&Uuid>) -> Option<Turn<'log>> {
        let turn = Turn { at, spoken };
        match spoken {
            Spoken::Lead(_) => Some(turn),
            Spoken::Owner { uuid, .. } => known.contains(uuid).then_some(turn),
        }
    }

    fn chars(&self) -> usize {
        match self.spoken {
            Spoken::Lead(said) | Spoken::Owner { said, .. } => AsRef::<str>::as_ref(said).len(),
        }
    }

    fn answering(&self, answer: &Uuid) -> Option<usize> {
        match self.spoken {
            Spoken::Owner { uuid, .. } => (uuid == answer).then_some(self.at),
            Spoken::Lead(_) => None,
        }
    }

    fn asking(&self, question: &Body) -> Option<usize> {
        match self.spoken {
            Spoken::Lead(said) => {
                AsRef::<str>::as_ref(said).contains(AsRef::<str>::as_ref(question)).then_some(self.at)
            }
            Spoken::Owner { .. } => None,
        }
    }
}

impl<'log> Excerpt<'log> {
    const CHARS: usize = 100_000;
    const SHORTEST: usize = 12;

    pub fn cut(turns: &[Turn<'log>]) -> Vec<Excerpt<'log>> {
        turns.iter().fold(Vec::new(), |mut excerpts: Vec<Excerpt<'log>>, turn| {
            let chars = turn.chars();
            let (closed, open) = match excerpts.pop() {
                Some(last) if last.chars.saturating_add(chars) <= Excerpt::CHARS => (None, last.joined(*turn, chars)),
                other => (other, Excerpt { turns: vec![*turn], chars }),
            };
            excerpts.extend(closed);
            excerpts.push(open);
            excerpts
        })
    }

    fn joined(self, turn: Turn<'log>, chars: usize) -> Excerpt<'log> {
        Excerpt { turns: self.turns.into_iter().chain([turn]).collect(), chars: self.chars.saturating_add(chars) }
    }

    pub fn verified(&self, heard: &Heard) -> BTreeSet<Pair> {
        let picked: Vec<Picked> = heard.reply().map_or_default(|reply| Replied::from(reply).listed());
        picked.into_iter().filter_map(|pick| self.located(pick)).collect()
    }

    fn located(&self, pick: Picked) -> Option<Pair> {
        let answered = self.turns.iter().position(|turn| turn.answering(&pick.answer_uuid).is_some())?;
        let before = self.turns.get(..answered)?;
        let at = before.iter().rev().find_map(|turn| turn.asking(&pick.question))?;
        let long = AsRef::<str>::as_ref(&pick.question).chars().count() >= Excerpt::SHORTEST;
        long.then_some(Pair { at, question: pick.question, answer_uuid: pick.answer_uuid })
    }
}

impl Pair {
    pub fn read(text: &str) -> Vec<Pair> {
        let mut pairs: Vec<Pair> = text
            .parse::<Lines>()
            .into_iter()
            .flatten()
            .filter_map(|line| serde_json::from_str::<Pair>(&line).ignoring(Ignored::Unlogged))
            .collect();
        pairs.sort();
        pairs
    }

    pub fn question(&self) -> &Body {
        &self.question
    }

    pub fn answer(&self) -> &Uuid {
        &self.answer_uuid
    }
}

impl<'pairs> From<&'pairs [Pair]> for PairLines<'pairs> {
    fn from(pairs: &'pairs [Pair]) -> PairLines<'pairs> {
        PairLines(pairs)
    }
}

impl fmt::Display for Excerpt<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Below is part of a conversation between an AI agent, the lead, and its owner. Each message is headed [lead] or [owner <uuid>].\n\
             List every question the lead asked the owner that a later owner message below answers.\n\
             Reply with JSON only, no prose and no code fence: a list of {{\"question\": the lead's question, one sentence copied exactly, character for character, \"answer_uuid\": the full uuid of the owner message that answers it}}. Reply [] if there are none.\n"
        )?;
        self.turns.iter().try_for_each(|turn| match turn.spoken {
            Spoken::Lead(said) => write!(formatter, "\n[lead]\n{said}\n"),
            Spoken::Owner { uuid, said } => write!(formatter, "\n[owner {uuid}]\n{said}\n"),
        })
    }
}

impl fmt::Display for PairLines<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|pair| {
            let line = serde_json::json!({
                "at": pair.at,
                "question": pair.question.to_string(),
                "answer_uuid": pair.answer_uuid.to_string(),
            });
            writeln!(formatter, "{line}")
        })
    }
}
