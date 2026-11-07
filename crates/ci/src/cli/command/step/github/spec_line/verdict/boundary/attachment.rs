use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub struct NotWellFormed;

#[derive(Debug, PartialEq, Eq)]
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

impl<'line> TryFrom<&'line str> for Attachment<'line> {
    type Error = NotWellFormed;

    fn try_from(value: &'line str) -> Result<Self, NotWellFormed> {
        Some(value)
            .filter(|value| *value == Self::NONE)
            .or_else(|| Self::owner(value).map(|_| value))
            .or_else(|| Self::labels(value).map(|_| value))
            .map(Attachment)
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
    const TRANSITIONS: [(Self, Mark, Self); 12] = [
        (Self::Gap, Mark::Lower, Self::Project),
        (Self::Gap, Mark::Digit, Self::Project),
        (Self::Gap, Mark::Dash, Self::Project),
        (Self::Project, Mark::Lower, Self::Project),
        (Self::Project, Mark::Digit, Self::Project),
        (Self::Project, Mark::Dash, Self::Project),
        (Self::Project, Mark::Hash, Self::Kind),
        (Self::Kind, Mark::KindLetter, Self::Number),
        (Self::Number, Mark::Digit, Self::Digits),
        (Self::Digits, Mark::Digit, Self::Digits),
        (Self::Gap, Mark::Space, Self::Gap),
        (Self::Digits, Mark::Comma, Self::Gap),
    ];

    fn after(self, letter: char) -> Option<Self> {
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
    const SIGNS: [(char, Self); 6] = [
        ('-', Self::Dash),
        ('#', Self::Hash),
        ('D', Self::KindLetter),
        ('L', Self::KindLetter),
        (' ', Self::Space),
        (',', Self::Comma),
    ];
}

impl From<char> for Mark {
    fn from(letter: char) -> Self {
        Some(letter)
            .filter(char::is_ascii_lowercase)
            .map(|_| Self::Lower)
            .or_else(|| Some(letter).filter(char::is_ascii_digit).map(|_| Self::Digit))
            .or_else(|| Self::SIGNS.into_iter().find(|(sign, _)| *sign == letter).map(|(_, mark)| mark))
            .unwrap_or(Self::Other)
    }
}
