use crate::event::{Event, Record};
use crate::ids::{EscapeId, EscapeKind, PrNumber, Reason, TaskId};
use crate::ledger::Ledger;
use crate::reply::{Accepted, Refusal};
use crate::spec::{SelfFile, Unreadable};
use crate::task::Progress;
use serde::{Deserialize, Serialize};

/// The escape kinds in `self/escapes.toml`, each with a source.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EscapeKinds {
    kind: Vec<Kind>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Kind {
    id: EscapeKind,
    text: String,
    source: String,
    /// Whether an escape of this kind names the PR merged outside meta that it records (P1).
    names_merged_pr: bool,
}

/// Whether an escape's kind was checked against `self/escapes.toml` when it was recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Checked {
    #[default]
    Listed,
    /// No escape kinds could be read without git or network; the kind is as given.
    Unchecked,
}

impl Checked {
    pub fn is_listed(&self) -> bool {
        *self == Self::Listed
    }
}

impl EscapeKinds {
    pub fn parse(text: &str) -> Result<Self, Unreadable> {
        SelfFile::EscapeKinds.parse(text)
    }

    fn admit(&self, kind: &EscapeKind) -> Result<(), Refusal> {
        match self.kind.iter().any(|known| known.id == *kind) {
            true => Ok(()),
            false => {
                let listed: String = self
                    .kind
                    .iter()
                    .map(|known| format!("  - {}: {}\n    source: {}\n", known.id, known.text, known.source))
                    .collect();
                Err(Refusal::because(format!(
                    "{kind} is not an escape kind. Next: meta escape <kind> <reason>, with one of the kinds in {}:\n{listed}",
                    SelfFile::EscapeKinds.path()
                )))
            }
        }
    }

    pub(crate) fn bypass(&self, kind: &EscapeKind) -> bool {
        *kind == EscapeKind::unclassified() || self.kind.iter().any(|known| known.id == *kind && known.names_merged_pr)
    }
}

/// One escape in the fold: its kind, its reason and the done task that resolved it, if any.
#[derive(Debug)]
pub struct Logged {
    kind: EscapeKind,
    reason: Reason,
    resolved: Option<TaskId>,
}

/// What repeats count by: the kind, or, for an unclassified escape, its free-text reason.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct RepeatKey<'a> {
    kind: &'a EscapeKind,
    reason: Option<&'a Reason>,
}

impl std::fmt::Display for RepeatKey<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.reason {
            Some(reason) => write!(formatter, "{} \"{reason}\"", self.kind),
            None => write!(formatter, "{}", self.kind),
        }
    }
}

impl Logged {
    pub(crate) fn new(kind: EscapeKind, reason: Reason) -> Self {
        Self { kind, reason, resolved: None }
    }

    pub(crate) fn resolve(&mut self, escape: EscapeId, task: TaskId) -> Result<(), String> {
        match self.resolved.replace(task) {
            None => Ok(()),
            Some(earlier) => Err(format!("{escape} resolved twice, against {earlier} and {task}")),
        }
    }

    pub(crate) fn is_open(&self) -> bool {
        self.resolved.is_none()
    }

    pub(crate) fn kind(&self) -> &EscapeKind {
        &self.kind
    }

    pub(crate) fn key(&self) -> RepeatKey<'_> {
        let reason = (self.kind == EscapeKind::unclassified()).then_some(&self.reason);
        RepeatKey { kind: &self.kind, reason }
    }

    pub(crate) fn line(&self, id: EscapeId) -> String {
        format!("{id} [{}] {}", self.kind, self.reason)
    }
}

#[derive(Debug)]
pub struct Escape {
    kind: EscapeKind,
    reason: Reason,
    pr: Option<PrNumber>,
}

impl Escape {
    pub fn new(kind: EscapeKind, reason: Reason, pr: Option<PrNumber>) -> Self {
        Self { kind, reason, pr }
    }

    /// Records the escape from the raw records, so that it works even on a log that does not fold.
    pub fn tally(self, kinds: &EscapeKinds, earlier: Result<&[Record], &str>) -> Result<Accepted, Refusal> {
        kinds.admit(&self.kind)?;
        let Self { kind, reason, pr } = self;
        let counts = earlier.map(|records| {
            records.iter().fold((0, 0, 0), |(escaped, resolved, repeats), record| match &record.event {
                Event::Escaped { kind: earlier, .. } => {
                    (escaped + 1, resolved, repeats + usize::from(*earlier == kind))
                }
                Event::Resolved { .. } => (escaped, resolved + 1, repeats),
                _ => (escaped, resolved, repeats),
            })
        });
        let text = match counts {
            Ok((escaped, resolved, repeats)) => format!(
                "{} recorded: [{kind}] {reason}\nopen escapes: {}\nrepeats of kind {kind}: {repeats}\n",
                EscapeId::after(escaped),
                (escaped + 1).saturating_sub(resolved),
            ),
            Err(why) => format!("escape recorded: [{kind}] {reason}\nid and counts unavailable: {why}\n"),
        };
        Ok(Accepted::new(vec![Event::Escaped { kind, reason, pr, checked: Checked::Listed }], text))
    }
}

impl Ledger {
    pub fn resolve(&self, escape: EscapeId, task: TaskId) -> Result<Accepted, Refusal> {
        let logged = self.escape(escape).ok_or_else(|| {
            Refusal::because(format!("{escape} is unknown. Next: meta status lists the open escapes."))
        })?;
        let refuse = |why: String| Err(Refusal::because(why));
        match (logged.resolved, self.tasks.get(&task)) {
            (Some(earlier), _) => refuse(format!("{escape} is already resolved against {earlier}.")),
            (None, Some(Progress::Done(_))) => {
                let text = format!("{escape} resolved against {task}\n");
                Ok(Accepted::new(vec![Event::Resolved { escape, task }], text))
            }
            (None, Some(other)) => refuse(format!(
                "{other}; an escape is resolved only against a done task. Next: merge the fix with meta merge, then meta resolve {escape} {task}."
            )),
            (None, None) => refuse(format!("{task} is unknown. Next: meta status.")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EscapeKinds;

    #[test]
    fn the_repo_escape_kinds_parse_each_with_a_source() {
        let kinds = EscapeKinds::parse(include_str!("../../../self/escapes.toml")).unwrap();
        assert!(kinds.kind.len() >= 2 && kinds.kind.iter().all(|kind| !kind.source.is_empty()));
        assert!(kinds.kind.iter().all(|kind| kind.id.to_string() != "unclassified"));
    }
}
