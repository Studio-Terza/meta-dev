use std::fmt;
use std::io::Write;

use gate_limits::Finding;

use super::number::Number;
use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Invocation, Ports};

#[derive(Debug)]
pub struct Ticket<'finding> {
    finding: &'finding Finding,
    number: Option<Number>,
}

impl<'finding> Ticket<'finding> {
    pub fn open_for(finding: &'finding Finding, number: Option<Number>) -> Ticket<'finding> {
        Ticket { finding, number }
    }

    pub fn filed(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure> {
        let stage = self.finding.stage();
        let invocation = if let Some(number) = self.number {
            writeln!(out, "{stage} is past its warn level: updating issue {number}")?;
            Invocation::new("gh", ["issue", "edit", &number.to_string(), "--body", &self.to_string()])
        } else {
            writeln!(out, "{stage} is past its warn level: opening an issue")?;
            Invocation::new("gh", ["issue", "create", "--title", stage.as_ref(), "--body", &self.to_string()])
        };
        ports.run(&invocation).map(|run| run.exit()).map_err(Failure::starting(&invocation))
    }
}

impl fmt::Display for Ticket<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.finding.fmt(formatter)
    }
}
