mod listed;
mod number;
mod title;

use std::fmt;

use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Invocation, Ports, Run, Text};
use listed::Listed;
use newtype::Wrapping;
use number::Number;
pub(crate) use title::Title;

#[derive(Debug)]
pub(crate) struct Issue<'title> {
    title: Title<'title>,
    number: Option<Number>,
}

impl<'title> Issue<'title> {
    const SEEN: &'static str = "100";

    pub(super) fn titled(titles: &[Title<'title>], ports: &impl Ports) -> Result<Vec<Issue<'title>>, Failure<'static>> {
        let invocations: Vec<Invocation> = titles.iter().map(|title| Self::search(*title)).collect();
        let texts = Failure::started(ports.outputs(&invocations), &invocations)?;
        titles.iter().zip(&texts).map(|(title, text)| Self::listed(*title, text)).collect()
    }

    pub(super) fn filed(filings: &[Invocation], ports: &impl Ports) -> Result<Exit, Failure<'static>> {
        let runs = Failure::started(ports.runs(filings), filings)?;
        Ok(runs.iter().map(Run::exit).find(|exit| *exit != Exit::Success).unwrap_or(Exit::Success))
    }

    pub(super) fn filing(&self, body: &impl fmt::Display) -> Invocation {
        match self.number {
            Some(number) => Invocation::new("gh", ["issue", "edit", &number.to_string(), "--body", &body.to_string()]),
            None => Invocation::new(
                "gh",
                ["issue", "create", "--title", self.title.into_inner(), "--body", &body.to_string()],
            ),
        }
    }

    pub(super) fn announcement(&self) -> Announcement<'_, 'title> {
        Announcement(self)
    }

    fn search(title: Title<'title>) -> Invocation {
        let search = Issue { title, number: None }.to_string();
        Invocation::new(
            "gh",
            ["issue", "list", "--state", "open", "--limit", Self::SEEN, "--search", &search, "--json", "number,title"],
        )
    }

    fn listed(title: Title<'title>, text: &Text) -> Result<Issue<'title>, Failure<'static>> {
        let listed: Vec<Listed> = serde_json::from_str(text.as_ref()).map_err(Failure::Issues)?;
        Ok(Issue { title, number: listed.iter().find_map(|issue| issue.numbered(title)) })
    }
}

impl fmt::Display for Issue<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "\"{}\" in:title", self.title)
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Announcement<'issue, 'title>(&'issue Issue<'title>);

impl fmt::Display for Announcement<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.number {
            Some(number) => writeln!(formatter, "updating issue {number}: {}", self.0.title),
            None => writeln!(formatter, "opening an issue: {}", self.0.title),
        }
    }
}
