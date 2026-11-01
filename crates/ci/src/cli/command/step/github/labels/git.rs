use text_lines::Lines;

use super::change::Change;
use crate::cli::command::failure::Failure;
use crate::cli::ports::{Invocation, Ports};

#[derive(Debug)]
pub struct Git<'ports, Host: Ports> {
    ports: &'ports Host,
}

impl<'ports, Host: Ports> Git<'ports, Host> {
    const REMOTE: &'static str = "origin/";
    const HEAD: &'static str = "HEAD";

    pub(crate) fn with_ports(ports: &'ports Host) -> Git<'ports, Host> {
        Git { ports }
    }

    pub(crate) fn changes_since_fork(&self, base: &str) -> Result<Vec<Change>, Failure> {
        let fork = self.fork_point(base)?;
        let listed = self.asked(&Invocation::new(
            "git",
            ["diff", "--name-status", "--find-renames", "--find-copies", fork.as_str(), Self::HEAD],
        ))?;
        Ok(Lines::from(listed.as_str()).filter_map(|line| line.parse::<Change>().ok()).collect())
    }

    fn fork_point(&self, base: &str) -> Result<String, Failure> {
        let remote = [Self::REMOTE, base].concat();
        let found = self.asked(&Invocation::new("git", ["merge-base", remote.as_str(), Self::HEAD]))?;
        Ok(Lines::from(found.as_str()).next().unwrap_or_default().to_owned())
    }

    fn asked(&self, invocation: &Invocation) -> Result<String, Failure> {
        self.ports.output(invocation).map_err(Failure::starting(invocation))
    }
}
