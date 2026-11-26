use std::collections::BTreeSet;
use std::fmt;
use std::ops::Not;

use design_model::{NodeId, Outline, Stage, Title};
use design_store::{Quote, Said, Unabridged};
use dispatch_ports::{Prompt, Reply};
use newtype::Wrapping;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use transcript_ports::{Body, Uuid};

use crate::cli::request::ignored::Ignored;
use invariants::Ignoring;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Role(&'static str);

#[derive(Debug, Deserialize)]
pub struct Asked {
    kind: Kind,
    question: Body,
}

#[derive(Debug)]
pub struct Reading<'history> {
    chunks: Vec<&'history [Said]>,
    prompts: Vec<Prompt>,
}

#[derive(Debug)]
pub struct Trimming<'asked> {
    question: &'asked Body,
    quotes: Vec<String>,
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
    read: usize,
    quoted: usize,
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
    shown: Vec<String>,
}

#[derive(Clone, Copy, Debug)]
pub struct Replied<'reply>(&'reply Reply);

#[derive(Debug)]
pub struct Recalling<'nodes> {
    said: &'nodes Body,
    nodes: &'nodes [Outline],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Remembered {
    node: NodeId,
    stage: Stage,
    title: Title,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Recollection(Vec<Remembered>);

impl<'nodes> Recalling<'nodes> {
    pub fn new(said: &'nodes Body, nodes: &'nodes [Outline]) -> Recalling<'nodes> {
        Recalling { said, nodes }
    }
}

impl Recollection {
    pub fn recalled(reply: &Reply, nodes: &[Outline]) -> Recollection {
        let named: BTreeSet<i64> = Replied(reply).listed::<i64>().into_iter().collect();
        Recollection(
            nodes
                .iter()
                .filter(|outline| named.contains(&outline.node().into_inner()))
                .filter_map(|outline| {
                    outline.title().map(|title| Remembered {
                        node: outline.node(),
                        stage: outline.stage(),
                        title: title.clone(),
                    })
                })
                .collect(),
        )
    }
}

impl AsRef<[Remembered]> for Recollection {
    fn as_ref(&self) -> &[Remembered] {
        &self.0
    }
}

impl fmt::Display for Recollection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|remembered| writeln!(formatter, "{remembered}"))
    }
}

impl fmt::Display for Remembered {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "n{} {}: {}", self.node.into_inner(), self.stage, self.title)
    }
}

impl fmt::Display for Recalling<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "The owner of a software project just sent its AI agent this message:\n{}\n\n\
             Below are the project's design nodes, one per line as n<number> <stage>: <title>.\n\
             List the numbers of every node that genuinely bears on the message: a node it asks about, decides, changes or depends on. Leave out nodes that only share a word with it.\n\
             Reply with JSON only, no prose and no code fence: a list of node numbers, such as [12, 40]. Reply [] if none bear on it.\n\n",
            self.said
        )?;
        self.nodes.iter().filter_map(|outline| outline.title().map(|title| (outline, title))).try_for_each(
            |(outline, title)| writeln!(formatter, "n{} {}: {}", outline.node().into_inner(), outline.stage(), title),
        )
    }
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
    pub fn new(asked: Asked, read: usize, quoted: usize, answers: Option<Answers>) -> Question {
        Question { asked, read, quoted, answers }
    }
}

impl fmt::Display for Question {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "question: {}", self.asked.question)?;
        writeln!(formatter, "read: {} owner messages", self.read)?;
        writeln!(formatter, "quoted: {}", self.quoted)?;
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
    pub fn read(reply: &Reply) -> Option<Asked> {
        Replied(reply).object::<Asked>().ignoring(Ignored::Unclassified).filter(|asked| asked.kind == Kind::Question)
    }

    fn selecting(&self, shown: &[impl fmt::Display]) -> Selecting<'_> {
        Selecting { question: &self.question, shown: shown.iter().map(ToString::to_string).collect() }
    }

    pub fn reading<'history>(&self, history: &'history [Said]) -> Reading<'history> {
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

    pub fn trimming(&self, quotes: &[Quote]) -> Trimming<'_> {
        Trimming {
            question: &self.question,
            quotes: quotes.iter().zip(1..).map(|(quote, number)| format!("{number}. {quote}")).collect(),
        }
    }
}

impl Reading<'_> {
    const CHUNK: usize = 300;

    pub fn prompts(&self) -> &[Prompt] {
        &self.prompts
    }

    pub fn quoted(&self, replies: &[Option<Reply>]) -> Vec<Vec<Quote>> {
        self.chunks
            .iter()
            .zip(replies)
            .map(|(chunk, read)| read.as_ref().map_or_default(|reply| Answers::quoted(reply, chunk)))
            .collect()
    }
}

impl AsRef<Body> for Asked {
    fn as_ref(&self) -> &Body {
        &self.question
    }
}

impl Replied<'_> {
    pub fn object<T: DeserializeOwned>(self) -> Result<T, serde_json::Error> {
        serde_json::from_str(&self.braced('{', '}'))
    }

    pub fn listed<T: DeserializeOwned>(self) -> Vec<T> {
        serde_json::from_str(&self.braced('[', ']')).ignoring(Ignored::Unpicked).unwrap_or_default()
    }

    pub fn of(reply: &Reply) -> Replied<'_> {
        Replied(reply)
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

    pub fn kept(reply: &Reply, quotes: &[Quote]) -> Option<Answers> {
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

impl fmt::Display for Classifying {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "You read the last message an AI agent wrote at the end of its turn. The agent is {}.\n\
             Decide whether the message asks its owner a question, or asks the owner to make a decision, approve something, or choose between options.\n\
             Reply with JSON only, no prose and no code fence:\n\
             {{\"kind\": \"question\" or \"statement\", \"question\": the question restated in one line (empty for a statement)}}\n\n\
             Message:\n{}",
            self.role.0, self.said
        )
    }
}

impl fmt::Display for Selecting<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "An AI agent is about to ask its owner this question:\n{}\n\n\
             Below are earlier messages from the owner, each headed by [uuid timestamp].\n\
             List every passage that answers the question or bears on it: a missed answer costs far more than an extra passage, so when unsure, include it.\n\
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
            "An AI agent is about to ask its owner this question:\n{}\n\n\
             Below, numbered, are passages the owner wrote earlier, each with its message uuid and timestamp.\n\
             Keep only the passages that answer the question: the owner already stated the decision, rule, preference or fact the question needs, so the agent could act without asking. Keep at most {}.\n\
             Reply with JSON only, no prose and no code fence: a list of passage numbers, such as [3, 1]. Reply [] if none answer it.\n\n{}",
            self.question,
            Answers::KEPT,
            self.quotes.join("\n")
        )
    }
}

impl fmt::Display for Answers {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "The transcript may already answer this; reread before asking:")?;
        self.0.iter().try_for_each(|quote| writeln!(formatter, "{quote}"))
    }
}
