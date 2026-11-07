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

    pub(crate) fn read(text: &'body str) -> Self {
        let line = Self::trimmed(text);
        Some(line)
            .filter(|line| *line == Self::REPAIR)
            .map(|_| Line::Repair)
            .or_else(|| {
                line.split_at_checked(Self::SPEC.len())
                    .filter(|(head, _)| *head == Self::SPEC)
                    .map(|(_, value)| Line::Spec(value))
            })
            .unwrap_or(Line::Other)
    }

    pub(crate) const fn spec(self) -> Option<&'body str> {
        match self {
            Line::Spec(value) => Some(value),
            Line::Repair | Line::Other => None,
        }
    }

    fn trimmed(text: &str) -> &str {
        let trailing: usize = text.chars().rev().take_while(|letter| letter.is_whitespace()).map(char::len_utf8).sum();
        text.get(..text.len() - trailing).unwrap_or_default()
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
