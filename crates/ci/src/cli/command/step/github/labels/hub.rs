mod pull_request;

use super::super::pull_number::PullNumber;
use crate::cli::command::failure::Failure;
use crate::cli::ports::{Invocation, Ports};
pub use pull_request::PullRequest;

#[derive(Debug)]
pub struct Hub<'ports, Host: Ports> {
    ports: &'ports Host,
}

impl<'ports, Host: Ports> Hub<'ports, Host> {
    pub(crate) fn with_ports(ports: &'ports Host) -> Hub<'ports, Host> {
        Hub { ports }
    }

    pub(crate) fn pull_request(&self, pull_request: &PullNumber) -> Result<PullRequest, Failure> {
        let kind = PullRequest::KIND;
        let invocation =
            Invocation::new("gh", ["pr", "view", pull_request.as_ref(), "--json", "title,labels", "--jq", kind]);
        self.ports.output(&invocation).map(PullRequest::listed).map_err(Failure::starting(&invocation))
    }
}
