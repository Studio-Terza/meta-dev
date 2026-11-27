use std::fmt;

use proc_macro2::{Ident, Span};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Line(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Fix(pub &'static str);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Problem(String);

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Name(String);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Count(usize);

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

    pub(crate) fn text(&self) -> &str {
        &self.0
    }
}

impl Count {
    pub(crate) fn counted(count: usize) -> Count {
        Count(count)
    }

    pub(crate) fn number(self) -> usize {
        self.0
    }

    pub(crate) fn incremented(self) -> Count {
        Count(self.0.saturating_add(1))
    }
}

impl PartialEq<str> for Name {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<Ident> for Name {
    fn eq(&self, ident: &Ident) -> bool {
        ident == &self.0
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
