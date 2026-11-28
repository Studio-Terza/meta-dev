mod boundary;

use std::path::Path;

use design_messages::PullNumber;
use dispatch_ports::{Clock, Shell, Timeout};
use file_ports::Files;
use store_ports::Store;

use super::super::super::Ground;
use super::super::super::crew::{Crew, Program};
use crate::cli::request::InstallStep;

#[derive(Debug)]
pub(crate) struct Installing<'used, 'crew, C: Clock, G: Shell> {
    crew: &'crew Crew<'crew, C, G>,
    directory: &'used Path,
    pull: PullNumber,
}

impl<'used, 'crew, C: Clock, G: Shell> Installing<'used, 'crew, C, G> {
    const CHECKOUT: &'static str = "install-";
    const QUICK: Timeout = Timeout::new(120);
    const BUILD: Timeout = Timeout::new(20 * 60);

    pub(crate) fn new(
        crew: &'crew Crew<'crew, C, G>,
        directory: &'used Path,
        pull: PullNumber,
    ) -> Installing<'used, 'crew, C, G> {
        Installing { crew, directory, pull }
    }

    pub(crate) fn installed<F: Files, S: Store>(&self, ground: &Ground<'_, F, S>) -> Result<(), InstallStep> {
        let checkout = self.crew.scratch().join(format!("{}{}", Self::CHECKOUT, self.pull));
        let path = checkout.to_string_lossy();
        ground.prepared(self.crew.scratch()).map_err(|_unmade| InstallStep::Checkout)?;
        self.ran(Program::Git, &["fetch", "origin", "main"], self.directory, Self::QUICK).ok_or(InstallStep::Fetch)?;
        self.ran(Program::Git, &["worktree", "add", "--detach", &path, "origin/main"], self.directory, Self::QUICK)
            .ok_or(InstallStep::Checkout)?;
        let built = self
            .ran(Program::Cargo, &["install", "--path", "crates/design", "--locked", "--force"], &checkout, Self::BUILD)
            .ok_or(InstallStep::Build);
        let removed = self
            .ran(Program::Git, &["worktree", "remove", "--force", &path], self.directory, Self::QUICK)
            .ok_or(InstallStep::Unchecked);
        built.and(removed)?;
        let workspace = ground.workspace().to_string_lossy();
        let printed = self
            .ran(Program::Design, &["--workspace", &workspace, "hook", "--settings"], self.directory, Self::QUICK)
            .ok_or(InstallStep::Print)?;
        Self::hooked(ground, &printed)
    }
}
