mod label_set;

use crate::cli::command::failure::Failure;
use crate::cli::ports::{Invocation, Ports};
pub use label_set::LabelSet;

#[derive(Debug)]
pub struct Hub<'ports, Host: Ports> {
    ports: &'ports Host,
}

impl<'ports, Host: Ports> Hub<'ports, Host> {
    pub(crate) fn with_ports(ports: &'ports Host) -> Hub<'ports, Host> {
        Hub { ports }
    }

    pub(crate) fn labels_of(&self, pull_request: &str) -> Result<LabelSet, Failure> {
        let invocation =
            Invocation::new("gh", ["pr", "view", pull_request, "--json", "labels", "--jq", ".labels[].name"]);
        self.ports.output(&invocation).map(LabelSet::listed).map_err(Failure::starting(&invocation))
    }
}
