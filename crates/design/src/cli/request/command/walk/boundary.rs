use std::collections::BTreeSet;
use std::fmt;
use std::slice;

use strum::VariantArray;

use serde::Deserialize;
use transcript_ports::{Body, Uuid};

use super::super::crew::Heard;
use super::super::hook::Replied;
use crate::cli::failure::Failure;
use crate::cli::request::ignored::Ignored;

#[derive(Debug, Deserialize)]
pub struct Tree {
    roots: Vec<Node>,
}

#[derive(Debug, Deserialize)]
pub struct Node {
    altitude: Altitude,
    domain: Body,
    label: Body,
    children: Vec<Node>,
    messages: Vec<Uuid>,
}

#[derive(strum::Display, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
enum Altitude {
    Ideal,
    Plan,
    Feature,
    Ruling,
}

#[derive(strum::Display, strum::VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "lowercase")]
pub enum Variant {
    Levels,
    Flat,
}

#[derive(Clone, Debug)]
pub struct Choosing<'asked, 'tree> {
    question: &'asked Body,
    children: Vec<&'tree Node>,
    variant: Variant,
}

#[derive(Debug)]
pub struct Retrieved<'tree>(BTreeSet<&'tree Uuid>);

impl Tree {
    pub fn read(text: &str) -> Result<Tree, Failure> {
        serde_json::from_str(text).map_err(Failure::Tree)
    }

    pub fn roots(&self) -> &[Node] {
        &self.roots
    }

    pub fn messages(&self) -> Retrieved<'_> {
        Retrieved::reached(self.nodes())
    }

    pub fn nodes(&self) -> Vec<&Node> {
        Node::every(&self.roots)
    }
}

impl Node {
    pub fn children(&self) -> &[Node] {
        &self.children
    }

    pub fn subtree(&self) -> Vec<&Node> {
        Node::every(slice::from_ref(self))
    }

    fn every(nodes: &[Node]) -> Vec<&Node> {
        nodes.iter().flat_map(|node| [node].into_iter().chain(Node::every(&node.children))).collect()
    }
}

impl Variant {
    pub fn named(word: &str) -> Result<Variant, Failure> {
        Variant::VARIANTS.iter().find(|variant| variant.to_string() == word).copied().ok_or(Failure::Usage)
    }
}

impl<'asked, 'tree> Choosing<'asked, 'tree> {
    pub fn new(question: &'asked Body, children: Vec<&'tree Node>, variant: Variant) -> Choosing<'asked, 'tree> {
        Choosing { question, children, variant }
    }

    pub fn chosen(&self, heard: &Heard) -> Vec<&'tree Node> {
        let numbers: Option<Vec<usize>> =
            heard.reply().and_then(|reply| Replied::from(reply).parsed(Ignored::Unchosen));
        numbers.map_or_else(
            || self.children.clone(),
            |named| {
                self.children
                    .iter()
                    .zip(1..)
                    .filter(|(_child, number)| named.contains(number))
                    .map(|(child, _number)| *child)
                    .collect()
            },
        )
    }

    fn asked(&self) -> &'static str {
        match self.variant {
            Variant::Levels => Choosing::LEVELS,
            Variant::Flat => Choosing::FLAT,
        }
    }

    const LEVELS: &'static str = "The owner's earlier messages are filed in a tree of topics. Below, each topic is named by one sentence the owner wrote, with its altitude (ideal, plan, feature or ruling) and its domain. Under each topic lie many more owner messages, on narrower matters than its sentence names.\n\
         Find where the owner messages that bear on the agent's message are filed: an earlier answer to anything it asks, or a decision, rule, preference or instruction about the work it reports or plans. List every topic whose domain or subject the agent's message touches, even loosely, since a topic's sentence names only part of what lies under it. A missed topic costs far more than an extra one, so when unsure, include it.\n\
         Reply with JSON only, no prose and no code fence: a list of topic numbers, such as [3, 1]. Reply [] only if the agent's message has nothing to do with any topic below.\n";

    const FLAT: &'static str = "The owner's earlier messages are filed in a tree of topics. Below, every topic is listed by number, named by one sentence the owner wrote, with its altitude (ideal, plan, feature or ruling) and its domain. A topic holds the owner messages filed under it and under every topic beneath it.\n\
         List the topics that bear directly on the agent's message: those holding an earlier answer to anything it asks, a decision, rule, preference or instruction about the work it reports or plans, or a standing rule about how the agent should work, report, test or decide that applies to what it is doing now. Prefer the most specific topics that bear; name a broad topic only when the message concerns all of it. A missed topic costs far more than an extra one, so include every specific topic that may hold such words, often ten or more.\n\
         Reply with JSON only, no prose and no code fence: a list of topic numbers, such as [12, 4]. Reply [] if none bear on it.\n";
}

impl<'tree> Retrieved<'tree> {
    pub fn reached(nodes: impl IntoIterator<Item = &'tree Node>) -> Retrieved<'tree> {
        Retrieved(nodes.into_iter().flat_map(|node| &node.messages).collect())
    }

    pub fn uuids(&self) -> &BTreeSet<&'tree Uuid> {
        &self.0
    }
}

impl fmt::Display for Choosing<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "An AI agent just ended its turn with this message to its owner:\n{}\n\n{}",
            self.question,
            self.asked()
        )?;
        self.children.iter().zip(1..).try_for_each(|(child, number)| {
            write!(formatter, "\n{number}. [{} | {}] {}", child.altitude, child.domain, child.label)
        })
    }
}

impl fmt::Display for Retrieved<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|uuid| writeln!(formatter, "{uuid}"))
    }
}
