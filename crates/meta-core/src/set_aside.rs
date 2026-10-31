use crate::escape::{Checked, Logged};
use crate::ids::{EscapeKind, PrNumber, Reason, Timestamp};
use serde::{Deserialize, Serialize};

/// An escape from a build not verified to be from main. It stays in the side file beside the real
/// log and never enters the log. Unknown fields are skipped, because a branch build may add some.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnverifiedEscape {
    at: Timestamp,
    kind: EscapeKind,
    reason: Reason,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pr: Option<PrNumber>,
    #[serde(default, skip_serializing_if = "Checked::is_listed")]
    checked: Checked,
}

impl UnverifiedEscape {
    pub(crate) fn new(at: Timestamp, kind: EscapeKind, reason: Reason, pr: Option<PrNumber>, checked: Checked) -> Self {
        Self { at, kind, reason, pr, checked }
    }

    fn logged(&self) -> Logged {
        Logged::new(self.kind.clone(), self.reason.clone(), self.pr, self.checked)
    }
}

/// What a side file holds. A line that cannot be read is only counted: a branch build's lines must
/// never stop a main build.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UnverifiedEscapes {
    escapes: Vec<UnverifiedEscape>,
    unreadable: usize,
}

impl UnverifiedEscapes {
    /// From the file's lines, each as read or `None` when it could not be.
    pub fn of(lines: impl IntoIterator<Item = Option<UnverifiedEscape>>) -> Self {
        lines.into_iter().fold(Self::default(), |mut read, line| {
            match line {
                Some(escape) => read.escapes.push(escape),
                None => read.unreadable += 1,
            }
            read
        })
    }

    /// The escapes in the form the ledger keeps: open, with no id.
    pub(crate) fn logged(&self) -> Vec<Logged> {
        self.escapes.iter().map(UnverifiedEscape::logged).collect()
    }

    /// The status line for what the file held, when it held anything.
    pub(crate) fn note(&self) -> String {
        match (self.escapes.len(), self.unreadable) {
            (0, 0) => String::new(),
            (read, unreadable) => format!(
                "unverified escapes, from builds not on main: {read}, counted in P2; {unreadable} unreadable lines in the side file\n"
            ),
        }
    }
}
