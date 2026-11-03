mod listed;
mod number;
mod title;

use std::fmt;
use std::io::Write;

use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Invocation, Ports};
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

    pub(super) fn titled(title: Title<'title>, ports: &impl Ports) -> Result<Issue<'title>, Failure> {
        let search = Issue { title, number: None }.to_string();
        let invocation = Invocation::new(
            "gh",
            ["issue", "list", "--state", "open", "--limit", Self::SEEN, "--search", &search, "--json", "number,title"],
        );
        let text = ports.output(&invocation).map_err(Failure::starting(&invocation))?;
        let listed: Vec<Listed> = serde_json::from_str(text.as_ref()).map_err(Failure::Issues)?;
        Ok(Issue { title, number: listed.iter().find_map(|issue| issue.numbered(title)) })
    }

    pub(super) fn filed(
        &self,
        body: &impl fmt::Display,
        ports: &impl Ports,
        out: &mut impl Write,
    ) -> Result<Exit, Failure> {
        let body = body.to_string();
        let invocation = if let Some(number) = self.number {
            writeln!(out, "updating issue {number}: {}", self.title)?;
            Invocation::new("gh", ["issue", "edit", &number.to_string(), "--body", &body])
        } else {
            writeln!(out, "opening an issue: {}", self.title)?;
            Invocation::new("gh", ["issue", "create", "--title", self.title.into_inner(), "--body", &body])
        };
        ports.run(&invocation).map(|run| run.exit()).map_err(Failure::starting(&invocation))
    }
}

impl fmt::Display for Issue<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "\"{}\" in:title", self.title)
    }
}
