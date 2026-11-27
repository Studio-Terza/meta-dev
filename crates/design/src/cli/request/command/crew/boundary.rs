use std::path::Path;

use dispatch_ports::{Call, Clock, Deadline, Dispatch, Ran, Shell, Timeout};
use invariants::Ignoring;

use super::Crew;
use crate::cli::request::ignored::Ignored;

impl<D: Dispatch, C: Clock, G: Shell> Crew<'_, D, C, G> {
    pub(crate) fn spoke(
        &self,
        program: &'static str,
        arguments: &[&str],
        directory: &Path,
        timeout: Timeout,
    ) -> Option<Ran> {
        let spoken = arguments.iter().map(|word| (*word).to_owned()).collect();
        let deadline = Deadline::after(self.clock.now(), timeout);
        let call = Call::new(program, spoken, directory.to_path_buf(), deadline);
        self.shell.ran(&call).ignoring(Ignored::Unrun)
    }
}
