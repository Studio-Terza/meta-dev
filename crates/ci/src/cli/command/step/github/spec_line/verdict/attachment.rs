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
        Some(letter).filter(|letter| letter.is_ascii_hexdigit() || *letter == Self::DASH)
    }

    fn owner(value: &str) -> Option<&str> {
        let (_, id) = value.split_at_checked(Self::OWNER.len()).filter(|(head, _)| *head == Self::OWNER)?;
        let start = id.get(..Self::SHORTEST_ID_LEN)?;
        (start.chars().all(|letter| letter.is_ascii_hexdigit())
            && id.chars().all(|letter| Self::id_letter(letter).is_some()))
        .then_some(id)
    }

    fn labels(value: &str) -> Option<Reading> {
        value.chars().try_fold(Reading::Gap, Reading::after).filter(|end| *end == Reading::Digits)
    }
}

impl<'line> TryFrom<&'line str> for Attachment<'line> {
    type Error = NotWellFormed;

    fn try_from(value: &'line str) -> Result<Attachment<'line>, NotWellFormed> {
        (value == Self::NONE || Self::owner(value).is_some() || Self::labels(value).is_some())
            .then_some(Attachment(value))
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
    fn after(self, letter: char) -> Option<Reading> {
        match (self, letter) {
            (Reading::Gap | Reading::Project, 'a'..='z' | '0'..='9' | '-') => Some(Reading::Project),
            (Reading::Project, '#') => Some(Reading::Kind),
            (Reading::Kind, 'D' | 'L') => Some(Reading::Number),
            (Reading::Number | Reading::Digits, '0'..='9') => Some(Reading::Digits),
            (Reading::Gap, ' ') | (Reading::Digits, ',') => Some(Reading::Gap),
            _ => None,
        }
    }
}
