use crate::escape::{Checked, Logged, described};
use crate::event::Event;
use crate::ids::{EscapeKind, PrNumber, Reason, Timestamp};
use crate::imported::Imported;
use crate::line_hash::LineHash;
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
        Logged::new(self.kind.clone(), self.reason.clone(), self.pr, self.checked, None)
    }

    /// The `Escaped` event that moves this escape into the log, marked as imported from the side
    /// file line `line`, and what to say about it.
    pub(crate) fn imported(&self, line: LineHash) -> (Event, String) {
        let event = Event::Escaped {
            kind: self.kind.clone(),
            reason: self.reason.clone(),
            pr: self.pr,
            checked: self.checked,
            imported: Some(Imported { line, written: self.at }),
        };
        (event, format!("{} (from an unverified build)", described(&self.kind, Checked::Listed, self.pr, &self.reason)))
    }
}

/// The text as one line a reader cannot mistake for several: a branch build wrote the side file, so
/// its text is not trusted. Line breaks and other control characters, and the backslash that
/// introduces an escape, show as escapes, and so do characters that reorder or hide text (bidi
/// controls, zero-width characters, the byte order mark).
pub(crate) fn one_line(text: &str) -> String {
    text.chars().map(shown).collect()
}

fn shown(character: char) -> String {
    match character {
        '\\' => "\\\\".to_owned(),
        '\n' => "\\n".to_owned(),
        '\r' => "\\r".to_owned(),
        '\t' => "\\t".to_owned(),
        other if other.is_control() || hides(other) => other.escape_unicode().to_string(),
        other => other.to_string(),
    }
}

/// Characters that are not control characters but reorder text or take no room.
fn hides(character: char) -> bool {
    matches!(
        character,
        '\u{2028}' | '\u{2029}' | '\u{200B}'..='\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2060}'..='\u{2064}'
            | '\u{2066}'..='\u{2069}' | '\u{FEFF}' | '\u{00AD}' | '\u{061C}'
    )
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

#[cfg(test)]
mod tests {
    use super::one_line;

    #[test]
    fn one_line_shows_bidi_and_zero_width_characters_as_escapes_and_keeps_ordinary_text() {
        let shown = one_line("a\u{202E}b\u{200B}c\u{2066}d\u{FEFF}e é 日本");
        assert_eq!(shown, r"a\u{202e}b\u{200b}c\u{2066}d\u{feff}e é 日本");
    }
}
