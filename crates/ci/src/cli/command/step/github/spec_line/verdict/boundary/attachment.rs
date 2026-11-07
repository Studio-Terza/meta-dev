use std::fmt;

use newtype::Checked;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("an attachment is none, an owner id, or a list of spec labels")]
pub struct NotWellFormed;

#[derive(newtype::Newtype, Debug, PartialEq, Eq)]
#[newtype(checked)]
pub struct Attachment<'line>(&'line str);

impl Attachment<'_> {
    const NONE: &'static str = "none";
    const OWNER: &'static str = "owner ";
    const DASH: char = '-';
    const SHORTEST_ID_LEN: usize = 8;

    fn id_letter(letter: char) -> Option<char> {
        Some(letter).filter(char::is_ascii_hexdigit).or_else(|| Some(letter).filter(|letter| *letter == Self::DASH))
    }

    fn owner(value: &str) -> Option<&str> {
        let (_, id) = value.split_at_checked(Self::OWNER.len()).filter(|(head, _)| *head == Self::OWNER)?;
        id.get(..Self::SHORTEST_ID_LEN)
            .filter(|start| start.chars().all(|letter| letter.is_ascii_hexdigit()))
            .and(Some(id))
            .filter(|id| id.chars().all(|letter| Self::id_letter(letter).is_some()))
    }

    fn labels(value: &str) -> Option<Reading> {
        value.chars().try_fold(Reading::Gap, Reading::after).filter(|end| *end == Reading::Digits)
    }
}

impl<'line> Checked for Attachment<'line> {
    type Error = NotWellFormed;

    fn checked<'held>(value: &'held &'line str) -> Result<&'held &'line str, NotWellFormed> {
        Some(value)
            .filter(|value| **value == Self::NONE)
            .or_else(|| Self::owner(value).map(|_| value))
            .or_else(|| Self::labels(value).map(|_| value))
            .ok_or(NotWellFormed)
    }
}

impl fmt::Display for Attachment<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Reading {
    Gap,
    Project,
    Kind,
    Number,
    Digits,
}

impl Reading {
    const TRANSITIONS: [(Reading, Mark, Reading); 12] = [
        (Reading::Gap, Mark::Lower, Reading::Project),
        (Reading::Gap, Mark::Digit, Reading::Project),
        (Reading::Gap, Mark::Dash, Reading::Project),
        (Reading::Project, Mark::Lower, Reading::Project),
        (Reading::Project, Mark::Digit, Reading::Project),
        (Reading::Project, Mark::Dash, Reading::Project),
        (Reading::Project, Mark::Hash, Reading::Kind),
        (Reading::Kind, Mark::KindLetter, Reading::Number),
        (Reading::Number, Mark::Digit, Reading::Digits),
        (Reading::Digits, Mark::Digit, Reading::Digits),
        (Reading::Gap, Mark::Space, Reading::Gap),
        (Reading::Digits, Mark::Comma, Reading::Gap),
    ];

    fn after(self, letter: char) -> Option<Reading> {
        let mark = Mark::from(letter);
        Self::TRANSITIONS
            .into_iter()
            .filter(|(from, _, _)| *from == self)
            .find(|(_, read, _)| *read == mark)
            .map(|(_, _, to)| to)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mark {
    Lower,
    Digit,
    Dash,
    Hash,
    KindLetter,
    Space,
    Comma,
    Other,
}

impl Mark {
    const SIGNS: [(char, Mark); 6] = [
        ('-', Mark::Dash),
        ('#', Mark::Hash),
        ('D', Mark::KindLetter),
        ('L', Mark::KindLetter),
        (' ', Mark::Space),
        (',', Mark::Comma),
    ];
}

impl From<char> for Mark {
    fn from(letter: char) -> Mark {
        Some(letter)
            .filter(char::is_ascii_lowercase)
            .map(|_| Mark::Lower)
            .or_else(|| Some(letter).filter(char::is_ascii_digit).map(|_| Mark::Digit))
            .or_else(|| Self::SIGNS.into_iter().find(|(sign, _)| *sign == letter).map(|(_, mark)| mark))
            .unwrap_or(Mark::Other)
    }
}
