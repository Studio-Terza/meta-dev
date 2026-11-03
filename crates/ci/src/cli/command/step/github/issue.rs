mod listed;
mod number;
mod title;

use std::fmt;
use std::io::Write;

use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Invocation, Ports, Run, Text};
use listed::Listed;
use newtype::Newtype;
use number::Number;
pub use title::Title;

#[derive(Debug)]
pub struct Issue<'title> {
    title: Title<'title>,
    number: Option<Number>,
}

impl<'title> Issue<'title> {
    const SEEN: &'static str = "100";

    pub(super) fn titled(titles: &[Title<'title>], ports: &impl Ports) -> Result<Vec<Issue<'title>>, Failure> {
        let invocations: Vec<Invocation> = titles.iter().map(|title| Self::search(*title)).collect();
        let texts = Failure::started(ports.outputs(&invocations), &invocations)?;
        titles.iter().zip(&texts).map(|(title, text)| Self::listed(*title, text)).collect()
    }

    pub(super) fn filed(filings: &[Invocation], ports: &impl Ports) -> Result<Exit, Failure> {
        let runs = Failure::started(ports.runs(filings), filings)?;
        Ok(runs.iter().map(Run::exit).find(|exit| *exit != Exit::Success).unwrap_or(Exit::Success))
    }

    pub(super) fn filing(&self, body: &impl fmt::Display, out: &mut impl Write) -> Result<Invocation, Failure> {
        let body = body.to_string();
        if let Some(number) = self.number {
            writeln!(out, "updating issue {number}: {}", self.title)?;
            Ok(Invocation::new("gh", ["issue", "edit", &number.to_string(), "--body", &body]))
        } else {
            writeln!(out, "opening an issue: {}", self.title)?;
            Ok(Invocation::new("gh", ["issue", "create", "--title", self.title.into_inner(), "--body", &body]))
        }
    }

    fn search(title: Title<'title>) -> Invocation {
        let search = Issue { title, number: None }.to_string();
        Invocation::new(
            "gh",
            ["issue", "list", "--state", "open", "--limit", Self::SEEN, "--search", &search, "--json", "number,title"],
        )
    }

    fn listed(title: Title<'title>, text: &Text) -> Result<Issue<'title>, Failure> {
        let listed: Vec<Listed> = serde_json::from_str(text.as_ref()).map_err(Failure::Issues)?;
        Ok(Issue { title, number: listed.iter().find_map(|issue| issue.numbered(title)) })
    }
}

impl fmt::Display for Issue<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "\"{}\" in:title", self.title)
    }
}
