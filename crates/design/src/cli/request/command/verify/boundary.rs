use std::fmt;

use serde::Deserialize;
use transcript_ports::Body;

use super::super::crew::Heard;
use super::super::hook::Replied;
use super::super::pairs::Pair;
use crate::cli::request::ignored::Ignored;

#[derive(Clone, Copy, Debug)]
pub struct Check<'run> {
    pair: &'run Pair,
    answer: &'run Body,
}

#[derive(Debug, Deserialize)]
struct Verdict {
    verdict: Answered,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Answered {
    Yes,
    No,
}

impl<'run> Check<'run> {
    pub fn new(pair: &'run Pair, answer: &'run Body) -> Check<'run> {
        Check { pair, answer }
    }

    pub fn pair(&self) -> &'run Pair {
        self.pair
    }

    pub fn judged(heard: &Heard) -> Option<Answered> {
        let verdict: Option<Verdict> = heard.reply().and_then(|reply| Replied::from(reply).object(Ignored::Unverified));
        verdict.map(|read| read.verdict)
    }
}

impl fmt::Display for Check<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "An AI agent asked its owner this question:\n{}\n\n\
             Later the owner wrote this message:\n{}\n\n\
             Does the owner's message answer the agent's question, directly or by a decision that settles it? A message that only touches the same subject, or answers some other question, does not.\n\
             Reply with JSON only, no prose and no code fence: {{\"verdict\": \"yes\"}} or {{\"verdict\": \"no\"}}.",
            self.pair.question(),
            self.answer
        )
    }
}
