use std::path::Path;

use design_messages::PullNumber;
use dispatch_ports::{Ran, Timeout};
use file_ports::{Bytes, Files, Kind};
use json_ports::{Fields, Json, Opaque};
use store_ports::Store;

use super::super::super::Ground;
use super::super::super::crew::{Asked, Cut, Tape};
use crate::cli::request::ignored::Ignored;
use crate::cli::request::{InstallStep, Kept};
use invariants::Ignoring;

#[derive(Clone, Copy, Debug)]
pub struct Installing {
    pull: PullNumber,
}

#[derive(Debug, serde::Deserialize)]
struct Printed {
    hooks: Opaque,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
struct Configured {
    #[serde(default)]
    hooks: Opaque,
    #[serde(flatten)]
    rest: Fields,
}

impl Installing {
    const GIT: &'static str = "git";
    const CARGO: &'static str = "cargo";
    const DESIGN: &'static str = "design";
    const CHECKOUT: &'static str = "install-";
    const HERE: &'static str = ".";
    const QUICK: Timeout = Timeout::new(120);
    const BUILD: Timeout = Timeout::new(20 * 60);

    pub fn new(pull: PullNumber) -> Installing {
        Installing { pull }
    }

    pub fn installed<F: Files, S: Store>(
        self,
        ground: &Ground<'_, F, S>,
        tape: &Tape<'_>,
    ) -> Result<Result<(), InstallStep>, Asked> {
        let prepared = ground.prepared(tape.scratch()).map_err(|_unmade| InstallStep::Checkout);
        let printed = match prepared {
            Ok(()) => Cut::settled(self.printed(tape, ground.workspace()))?,
            Err(step) => Err(step),
        };
        Ok(printed.and_then(|hooks| Self::hooked(ground, hooks).ok_or(InstallStep::Settings)))
    }

    fn printed(self, tape: &Tape<'_>, workspace: &Path) -> Result<Opaque, Cut<InstallStep>> {
        let checkout = Kept::Agents.within().join(format!("{}{}", Self::CHECKOUT, self.pull));
        let path = checkout.to_string_lossy();
        let quick = |program, arguments: &[&str], step| {
            Self::ran(tape, program, arguments, workspace, Self::QUICK)?.ok_or(Cut::Failed(step))
        };
        quick(Self::GIT, &["fetch", "origin", "main"], InstallStep::Fetch)?;
        quick(Self::GIT, &["worktree", "add", "--detach", &path, "origin/main"], InstallStep::Checkout)?;
        let installed = ["install", "--path", "crates/design", "--locked", "--force"];
        let built = Self::ran(tape, Self::CARGO, &installed, &workspace.join(&checkout), Self::BUILD)?
            .ok_or(InstallStep::Build);
        let removed = Self::ran(tape, Self::GIT, &["worktree", "remove", "--force", &path], workspace, Self::QUICK)?;
        built.and(removed.ok_or(InstallStep::Unchecked)).map_err(Cut::Failed)?;
        let printed = quick(Self::DESIGN, &["--workspace", Self::HERE, "hook", "--settings"], InstallStep::Print)?;
        let settings: Printed = Json::read(&printed).map_err(|_unprinted| Cut::Failed(InstallStep::Print))?;
        Ok(settings.hooks)
    }

    fn ran(
        tape: &Tape<'_>,
        program: &'static str,
        arguments: &[&str],
        directory: &Path,
        timeout: Timeout,
    ) -> Result<Option<String>, Asked> {
        let ran = tape.spoke(program, arguments, directory, timeout)?;
        Ok(ran.as_ref().and_then(Ran::succeeded).map(|printed| AsRef::<str>::as_ref(printed).to_owned()))
    }

    fn hooked<F: Files, S: Store>(ground: &Ground<'_, F, S>, hooks: Opaque) -> Option<()> {
        let file = ground.settings();
        let configured: Configured = match ground.kind(&file) {
            Kind::Missing => Configured::default(),
            Kind::File | Kind::Directory | Kind::Link => {
                let text = ground.files().read_text(&file).ignoring(Ignored::Unhooked)?;
                Json::read(text.as_ref()).ignoring(Ignored::Unhooked)?
            }
        };
        let written = Json::pretty(&Configured { hooks, rest: configured.rest }).ignoring(Ignored::Unhooked)?;
        ground.prepared(file.parent()?).ignoring(Ignored::Unhooked)?;
        ground.files().write(&file, Bytes::new(format!("{written}\n").into_bytes())).ignoring(Ignored::Unhooked)
    }
}
