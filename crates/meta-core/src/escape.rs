use crate::event::{Event, Record};
use crate::ids::{EscapeId, EscapeKind, PrNumber, Reason, TaskId};
use crate::ledger::Ledger;
use crate::merges::Unreached;
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
    /// Whether an escape of this kind names, with `--pr`, the PR merged outside meta that it records (P1).
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

    fn admit(&self, kind: &EscapeKind, pr: Option<PrNumber>) -> Result<(), Refusal> {
        let known = self.kind.iter().find(|known| known.id == *kind).ok_or_else(|| {
            let listed: String = self
                .kind
                .iter()
                .map(|known| format!("  - {}: {}\n    source: {}\n", known.id, known.text, known.source))
                .collect();
            Refusal::because(format!(
                "{kind} is not an escape kind. Next: meta escape <kind> <reason>, with one of the kinds in {}:\n{listed}",
                SelfFile::EscapeKinds.path()
            ))
        })?;
        match (known.names_merged_pr, pr) {
            (true, Some(_)) | (false, None) => Ok(()),
            (true, None) => Err(Refusal::because(format!(
                "{kind} names the PR merged outside meta. Next: meta escape {kind} --pr <pr> <reason>."
            ))),
            (false, Some(pr)) => Err(Refusal::because(format!(
                "{kind} names no PR, so {pr} is not recorded. Next: meta escape {kind} <reason>."
            ))),
        }
    }

    /// Whether `kind` is one of these kinds.
    pub(crate) fn lists(&self, kind: &EscapeKind) -> bool {
        self.kind.iter().any(|known| known.id == *kind)
    }

    /// Whether an escape of this kind names a PR merged outside meta. An unclassified one never does.
    pub(crate) fn names_merged_pr(&self, kind: &EscapeKind) -> bool {
        self.kind.iter().any(|known| known.id == *kind && known.names_merged_pr)
    }
}

/// One escape in the fold: its kind, reason and PR, and the done task that resolved it, if any.
#[derive(Debug)]
pub struct Logged {
    kind: EscapeKind,
    reason: Reason,
    pr: Option<PrNumber>,
    checked: Checked,
    resolved: Option<TaskId>,
}

/// What repeats count by: the kind; an unclassified escape's free-text reason; or, for every kind
/// that main does not list, one shared key, so that a typo cannot dodge the bound.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum RepeatKey<'a> {
    Kind(&'a EscapeKind),
    Unclassified(&'a Reason),
    Unlisted,
}

impl std::fmt::Display for RepeatKey<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Kind(kind) => write!(formatter, "{kind}"),
            Self::Unclassified(reason) => write!(formatter, "{} \"{reason}\"", EscapeKind::unclassified()),
            Self::Unlisted => formatter.write_str("kinds not on main, unchecked"),
        }
    }
}

impl Logged {
    pub(crate) fn new(kind: EscapeKind, reason: Reason, pr: Option<PrNumber>, checked: Checked) -> Self {
        Self { kind, reason, pr, checked, resolved: None }
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

    /// The PR merged outside meta that this escape records, when its kind names one.
    pub(crate) fn merged_pr(&self, kinds: &EscapeKinds) -> Option<PrNumber> {
        self.pr.filter(|_| kinds.names_merged_pr(&self.kind))
    }

    /// Whether main's kinds, when they could be read, leave this kind out. Unclassified never is.
    fn off_main(&self, main: Option<&EscapeKinds>) -> bool {
        self.kind != EscapeKind::unclassified() && main.is_some_and(|kinds| !kinds.lists(&self.kind))
    }

    pub(crate) fn key(&self, main: Option<&EscapeKinds>) -> RepeatKey<'_> {
        match (self.kind == EscapeKind::unclassified(), self.off_main(main)) {
            (true, _) => RepeatKey::Unclassified(&self.reason),
            (false, true) => RepeatKey::Unlisted,
            (false, false) => RepeatKey::Kind(&self.kind),
        }
    }

    /// Unchecked when it was recorded so, or when its kind is not on main.
    pub(crate) fn line(&self, id: EscapeId, main: Option<&EscapeKinds>) -> String {
        let checked = match self.off_main(main) {
            true => Checked::Unchecked,
            false => self.checked,
        };
        format!("{id} {}", described(&self.kind, checked, self.pr, &self.reason))
    }
}

/// `[kind] #pr reason`, with `unchecked` beside a kind that was not checked.
fn described(kind: &EscapeKind, checked: Checked, pr: Option<PrNumber>, reason: &Reason) -> String {
    let mark = match checked {
        Checked::Listed => "",
        Checked::Unchecked => ", unchecked",
    };
    let pr = pr.map_or_else(String::new, |pr| format!("{pr} "));
    format!("[{kind}{mark}] {pr}{reason}")
}

#[derive(Debug)]
pub struct Escape {
    kind: EscapeKind,
    reason: Reason,
    pr: Option<PrNumber>,
}

/// An escape whose kind was checked against the kinds, or why no kinds could be read.
#[derive(Debug)]
pub struct Admitted {
    escape: Escape,
    unchecked: Option<Unreached>,
}

impl Escape {
    pub fn new(kind: EscapeKind, reason: Reason, pr: Option<PrNumber>) -> Self {
        Self { kind, reason, pr }
    }

    /// Checks the kind against `kinds`, read without git or network. Unreadable kinds leave it unchecked.
    pub fn admit(self, kinds: Result<EscapeKinds, Unreached>) -> Result<Admitted, Refusal> {
        match kinds {
            Ok(kinds) => kinds.admit(&self.kind, self.pr).map(|()| Admitted { escape: self, unchecked: None }),
            Err(_) if self.kind == EscapeKind::unclassified() => Err(Refusal::because(format!(
                "{} marks escapes logged before kinds existed and cannot be chosen. Next: meta escape <kind> <reason>.",
                self.kind
            ))),
            Err(why) => Ok(Admitted { escape: self, unchecked: Some(why) }),
        }
    }
}

impl Admitted {
    /// Records the escape from the raw records read under the log's lock, so that it works even on
    /// a log that does not fold; a log that does not parse leaves the id and counts unknown.
    pub fn tally(self, earlier: Result<&[Record], String>) -> Accepted {
        let Self { escape: Escape { kind, reason, pr }, unchecked } = self;
        let checked = unchecked.as_ref().map_or(Checked::Listed, |_| Checked::Unchecked);
        let counts = earlier.map(|records| {
            records.iter().fold((0, 0, 0), |(escaped, resolved, repeats), record| match &record.event {
                Event::Escaped { kind: earlier, .. } => {
                    (escaped + 1, resolved, repeats + usize::from(*earlier == kind))
                }
                Event::Resolved { .. } => (escaped, resolved + 1, repeats),
                _ => (escaped, resolved, repeats),
            })
        });
        let what = described(&kind, Checked::Listed, pr, &reason);
        let counted = match counts {
            Ok((escaped, resolved, repeats)) => format!(
                "{} recorded: {what}\nopen escapes: {}\nrepeats of kind {kind}: {repeats}\n",
                EscapeId::after(escaped),
                (escaped + 1).saturating_sub(resolved),
            ),
            Err(why) => format!("escape recorded: {what}\nid and counts unavailable: {why}\n"),
        };
        let unchecked = unchecked.map_or_else(String::new, |why| format!("kind unchecked: {why}\n"));
        Accepted::new(vec![Event::Escaped { kind, reason, pr, checked }], counted + &unchecked)
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
    fn the_repo_escape_kinds_parse_each_with_a_source_and_one_names_merged_prs() {
        let kinds = EscapeKinds::parse(include_str!("../../../self/escapes.toml")).unwrap();
        assert!(kinds.kind.len() >= 2 && kinds.kind.iter().all(|kind| !kind.source.is_empty()));
        assert!(kinds.kind.iter().all(|kind| kind.id.to_string() != "unclassified"));
        let naming: Vec<String> =
            kinds.kind.iter().filter(|kind| kind.names_merged_pr).map(|kind| kind.id.to_string()).collect();
        assert_eq!(naming, ["outside-merge"]);
    }
}
