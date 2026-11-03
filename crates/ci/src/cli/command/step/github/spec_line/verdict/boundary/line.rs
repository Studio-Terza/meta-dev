use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Line<'body> {
    Repair,
    Spec(&'body str),
    Other,
}

impl<'body> Line<'body> {
    const SPEC: &'static str = "spec: ";
    const REPAIR: &'static str = "repair: main";

    pub(crate) fn read(text: &'body str) -> Line<'body> {
        let line = Self::trimmed(text);
        match line.split_at_checked(Self::SPEC.len()) {
            _ if line == Self::REPAIR => Line::Repair,
            Some((head, value)) if head == Self::SPEC => Line::Spec(value),
            _ => Line::Other,
        }
    }

    pub(crate) fn spec(self) -> Option<&'body str> {
        match self {
            Line::Spec(value) => Some(value),
            Line::Repair | Line::Other => None,
        }
    }

    fn trimmed(text: &str) -> &str {
        let end = text
            .char_indices()
            .rev()
            .find(|(_, letter)| !letter.is_whitespace())
            .map_or(0, |(at, letter)| at + letter.len_utf8());
        text.get(..end).unwrap_or_default()
    }
}

impl fmt::Display for Line<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Line::Repair => formatter.write_str(Self::REPAIR),
            Line::Spec(value) => write!(formatter, "{}{value}", Self::SPEC),
            Line::Other => Ok(()),
        }
    }
}
