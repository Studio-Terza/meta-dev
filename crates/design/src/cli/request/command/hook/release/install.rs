use std::path::Path;

use design_messages::PullNumber;
use dispatch_ports::{Clock, Ran, Shell, Timeout};
use file_ports::{Bytes, Files, Kind};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use store_ports::Store;

use super::super::super::Ground;
use super::super::super::crew::Crew;
use crate::cli::request::InstallStep;
use crate::cli::request::ignored::Ignored;
use invariants::Ignoring;

#[derive(Debug)]
pub struct Installing<'used, 'crew, C: Clock, G: Shell> {
    crew: &'crew Crew<'crew, C, G>,
    directory: &'used Path,
    pull: PullNumber,
}

#[derive(Debug, Deserialize)]
struct Printed {
    hooks: Value,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct Configured {
    #[serde(default)]
    hooks: Value,
    #[serde(flatten)]
    rest: Map<String, Value>,
}

impl<'used, 'crew, C: Clock, G: Shell> Installing<'used, 'crew, C, G> {
    const GIT: &'static str = "git";
    const CARGO: &'static str = "cargo";
    const DESIGN: &'static str = "design";
    const CHECKOUT: &'static str = "install-";
    const QUICK: Timeout = Timeout::new(120);
    const BUILD: Timeout = Timeout::new(20 * 60);

    pub fn new(
        crew: &'crew Crew<'crew, C, G>,
        directory: &'used Path,
        pull: PullNumber,
    ) -> Installing<'used, 'crew, C, G> {
        Installing { crew, directory, pull }
    }

    pub fn installed<F: Files, S: Store>(&self, ground: &Ground<'_, F, S>) -> Result<(), InstallStep> {
        let checkout = self.crew.scratch().join(format!("{}{}", Self::CHECKOUT, self.pull));
        let path = checkout.to_string_lossy();
        ground.prepared(self.crew.scratch()).map_err(|_unmade| InstallStep::Checkout)?;
        self.ran(Self::GIT, &["fetch", "origin", "main"], self.directory, Self::QUICK).ok_or(InstallStep::Fetch)?;
        self.ran(Self::GIT, &["worktree", "add", "--detach", &path, "origin/main"], self.directory, Self::QUICK)
            .ok_or(InstallStep::Checkout)?;
        let built = self
            .ran(Self::CARGO, &["install", "--path", "crates/design", "--locked", "--force"], &checkout, Self::BUILD)
            .ok_or(InstallStep::Build);
        let removed = self
            .ran(Self::GIT, &["worktree", "remove", "--force", &path], self.directory, Self::QUICK)
            .ok_or(InstallStep::Unchecked);
        built.and(removed)?;
        let workspace = ground.workspace().to_string_lossy();
        let printed = self
            .ran(Self::DESIGN, &["--workspace", &workspace, "hook", "--settings"], self.directory, Self::QUICK)
            .ok_or(InstallStep::Print)?;
        let settings: Printed = serde_json::from_str(&printed).map_err(|_unprinted| InstallStep::Print)?;
        Self::hooked(ground, settings.hooks).ok_or(InstallStep::Settings)
    }

    fn ran(&self, program: &'static str, arguments: &[&str], directory: &Path, timeout: Timeout) -> Option<String> {
        let ran = self.crew.spoke(program, arguments, directory, timeout);
        ran.as_ref().and_then(Ran::succeeded).map(|printed| AsRef::<str>::as_ref(printed).to_owned())
    }

    fn hooked<F: Files, S: Store>(ground: &Ground<'_, F, S>, hooks: Value) -> Option<()> {
        let file = ground.settings();
        let configured: Configured = match ground.kind(&file) {
            Kind::Missing => Configured::default(),
            Kind::File | Kind::Directory | Kind::Link => {
                let text = ground.files().read_text(&file).ignoring(Ignored::Unhooked)?;
                serde_json::from_str(text.as_ref()).ignoring(Ignored::Unhooked)?
            }
        };
        let written =
            serde_json::to_string_pretty(&Configured { hooks, rest: configured.rest }).ignoring(Ignored::Unhooked)?;
        ground.prepared(file.parent()?).ignoring(Ignored::Unhooked)?;
        ground.files().write(&file, Bytes::new(format!("{written}\n").into_bytes())).ignoring(Ignored::Unhooked)
    }
}
