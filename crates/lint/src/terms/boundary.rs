use std::fmt;

use proc_macro2::{Ident, Span};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Line(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Fix(pub &'static str);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Word(pub &'static str);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Problem(String);

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Name(String);

#[derive(Debug)]
pub(crate) struct Text(String);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Count(usize);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Depth(usize);

impl Line {
    pub(crate) fn of(span: Span) -> Line {
        Line(span.start().line)
    }

    pub(crate) fn numbered(line: usize) -> Line {
        Line(line)
    }
}

impl Problem {
    pub(crate) fn said(problem: impl fmt::Display) -> Problem {
        Problem(problem.to_string())
    }
}

impl Name {
    pub(crate) fn of(ident: &Ident) -> Name {
        Name(ident.to_string())
    }

    pub(crate) fn spelled(text: &str) -> Name {
        Name(text.to_owned())
    }

    pub(crate) fn text(&self) -> &str {
        &self.0
    }

    pub(crate) fn as_crate(&self) -> Name {
        Name(self.0.chars().map(|letter| if letter == '-' { '_' } else { letter }).collect())
    }
}

impl Text {
    pub(crate) fn read(text: String) -> Text {
        Text(text)
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn lines(&self) -> Count {
        Count(self.0.bytes().filter(|byte| *byte == b'\n').count())
    }
}

impl Count {
    pub(crate) fn number(self) -> usize {
        self.0
    }

    pub(crate) fn incremented(self) -> Count {
        Count(self.0.saturating_add(1))
    }

    pub(crate) fn spanning(span: Span) -> Count {
        Count(span.end().line.saturating_sub(span.start().line).saturating_add(1))
    }

    pub(crate) fn plus(self, other: Count) -> Count {
        Count(self.0.saturating_add(other.0))
    }

    pub(crate) fn over_share(self, whole: Count, percent: usize) -> Option<()> {
        const WHOLE: usize = 100;
        (self.0.saturating_mul(WHOLE) > whole.0.saturating_mul(percent)).then_some(())
    }
}

impl Depth {
    pub(crate) fn deeper(self) -> Depth {
        Depth(self.0.saturating_add(1))
    }

    pub(crate) fn shallower(self) -> Depth {
        Depth(self.0.saturating_sub(1))
    }

    pub(crate) fn surface(self) -> Option<()> {
        (self.0 == 0).then_some(())
    }
}

impl AsRef<str> for Word {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl PartialEq<str> for Name {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl fmt::Display for Line {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl fmt::Display for Fix {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

impl fmt::Display for Problem {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl fmt::Display for Name {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl fmt::Display for Count {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
