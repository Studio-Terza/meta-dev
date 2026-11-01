mod listed;

use std::fmt;

use gate_limits::Stage;

use super::number::Number;
use crate::cli::command::failure::Failure;
use crate::cli::ports::{Invocation, Ports};
use listed::Listed;

#[derive(Debug)]
pub struct Search<'stage>(&'stage Stage);

impl<'stage> Search<'stage> {
    const SEEN: &'static str = "100";

    pub fn for_stage(stage: &'stage Stage) -> Search<'stage> {
        Search(stage)
    }

    pub fn number_among(&self, ports: &impl Ports) -> Result<Option<Number>, Failure> {
        let invocation = Invocation::new(
            "gh",
            [
                "issue",
                "list",
                "--state",
                "open",
                "--limit",
                Self::SEEN,
                "--search",
                &self.to_string(),
                "--json",
                "number,title",
            ],
        );
        let text = ports.output(&invocation).map_err(Failure::starting(&invocation))?;
        let listed: Vec<Listed> = serde_json::from_str(&text).map_err(Failure::Issues)?;
        Ok(listed.iter().find_map(|issue| issue.numbered(self.0)))
    }
}

impl fmt::Display for Search<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "\"{}\" in:title", self.0)
    }
}
