use std::collections::BTreeSet;

use grammar_hunk::Patch;
use serde::Deserialize;
use serde::de::IgnoredAny;
use text_lines::{Lines, OneLine};

use crate::cli::ports::Text;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
struct Function {
    function_name: OneLine,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
struct Position {
    line: u32,
    column: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
struct Span {
    start: Position,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub(in super::super) struct Mutant {
    name: OneLine,
    file: OneLine,
    function: Option<Function>,
    span: Span,
    replacement: OneLine,
    genre: OneLine,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Place {
    file: OneLine,
    line: u32,
    column: u32,
    function: Option<Function>,
    replacement: OneLine,
    genre: OneLine,
}

#[derive(Debug, Default)]
pub(in super::super) struct Places(BTreeSet<Place>);

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Scenario {
    Mutant {
        #[serde(rename = "Mutant")]
        mutant: Mutant,
    },
    Baseline(IgnoredAny),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
enum Summary {
    MissedMutant,
    Timeout,
    #[serde(other)]
    Other,
}
impl Summary {
    fn unproven(self) -> Option<()> {
        match self {
            Summary::MissedMutant | Summary::Timeout => Some(()),
            Summary::Other => None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct Outcome {
    scenario: Scenario,
    summary: Summary,
}

#[derive(Debug, Deserialize)]
pub(in super::super) struct Outcomes {
    outcomes: Vec<Outcome>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in super::super) struct Tree(String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in super::super) struct Head(String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in super::super) struct Commits {
    tree: Tree,
    head: Head,
}

impl Mutant {
    pub(in super::super) fn name(&self) -> Text {
        Text::new(self.name.to_string())
    }

    pub(in super::super) fn listed(text: &Text) -> Result<Vec<Mutant>, serde_json::Error> {
        serde_json::from_str(text.as_ref())
    }

    fn place(&self, line: u32) -> Place {
        Place {
            file: self.file.clone(),
            line,
            column: self.span.start.column,
            function: self.function.clone(),
            replacement: self.replacement.clone(),
            genre: self.genre.clone(),
        }
    }

    fn moved(&self, patch: &Patch) -> Option<Place> {
        let line = self.span.start.line;
        let file = Some(self.file.as_ref());
        let changed = patch.changed().iter().find(|changed| changed.before() == file);
        changed.map_or(Some(line), |found| found.moved(line)).map(|moved| self.place(moved))
    }
}

impl Places {
    pub(in super::super) fn placed(mutants: &[Mutant]) -> Places {
        Places(mutants.iter().map(|mutant| mutant.place(mutant.span.start.line)).collect())
    }

    pub(in super::super) fn moved(mutants: &[Mutant], patch: &Patch) -> Places {
        Places(mutants.iter().filter_map(|mutant| mutant.moved(patch)).collect())
    }

    pub(in super::super) fn either(&self, other: &Places, mutants: &[Mutant]) -> Vec<Text> {
        mutants
            .iter()
            .filter(|mutant| {
                let place = mutant.place(mutant.span.start.line);
                self.0.contains(&place).then_some(()).or(other.0.contains(&place).then_some(())).is_some()
            })
            .map(Mutant::name)
            .collect()
    }
}

impl Scenario {
    fn mutant(self) -> Option<Mutant> {
        match self {
            Scenario::Mutant { mutant } => Some(mutant),
            Scenario::Baseline(_) => None,
        }
    }
}

impl Outcomes {
    pub(in super::super) fn read(text: &Text) -> Result<Outcomes, serde_json::Error> {
        serde_json::from_str(text.as_ref())
    }

    pub(in super::super) fn missed(self) -> Vec<Mutant> {
        self.outcomes
            .into_iter()
            .filter(|outcome| outcome.summary.unproven().is_some())
            .filter_map(|outcome| outcome.scenario.mutant())
            .collect()
    }
}

impl Commits {
    pub(in super::super) fn read(text: &Text) -> Option<Commits> {
        let lines: Vec<String> = text.as_ref().parse::<Lines>().into_iter().flatten().collect();
        let [tree, head] = <[String; 2]>::try_from(lines).ok()?;
        Some(Commits { tree: Tree(tree), head: Head(head) })
    }

    pub(in super::super) fn tree(&self) -> &Tree {
        &self.tree
    }

    pub(in super::super) fn head(&self) -> &Head {
        &self.head
    }
}

impl AsRef<str> for Tree {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for Head {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
