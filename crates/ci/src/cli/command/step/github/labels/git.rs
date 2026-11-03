use text_lines::Lines;

use super::change::Change;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::branch::Branch;
use crate::cli::command::step::revision::Revision;
use crate::cli::ports::{Invocation, Ports, Text};

#[derive(Debug)]
pub struct Git<'ports, Host: Ports> {
    ports: &'ports Host,
}

impl<'ports, Host: Ports> Git<'ports, Host> {
    const REMOTE: &'static str = "origin/";
    const HEAD: &'static str = "HEAD";
    const LITERAL_PATHS: &'static str = "core.quotePath=false";

    pub(crate) fn with_ports(ports: &'ports Host) -> Git<'ports, Host> {
        Git { ports }
    }

    pub(crate) fn changes_since_fork(&self, base: &Branch) -> Result<Vec<Change>, Failure> {
        let fork = self.fork_point(base)?;
        let listed = self.asked(&Invocation::new(
            "git",
            [
                "-c",
                Self::LITERAL_PATHS,
                "diff",
                "--name-status",
                "--find-renames",
                "--find-copies",
                fork.as_ref(),
                Self::HEAD,
            ],
        ))?;
        Ok(Lines::from(listed.as_ref()).filter_map(|line| line.parse::<Change>().ok()).collect())
    }

    fn fork_point(&self, base: &Branch) -> Result<Revision, Failure> {
        let remote = [Self::REMOTE, base.as_ref()].concat();
        let found = self.asked(&Invocation::new("git", ["merge-base", remote.as_str(), Self::HEAD]))?;
        Ok(Revision::new(Lines::from(found.as_ref()).next().unwrap_or_default().to_owned()))
    }

    fn asked(&self, invocation: &Invocation) -> Result<Text, Failure> {
        self.ports.output(invocation).map_err(Failure::starting(invocation))
    }
}
