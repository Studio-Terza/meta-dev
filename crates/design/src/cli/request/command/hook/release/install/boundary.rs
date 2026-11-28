use std::path::Path;

use dispatch_ports::{Clock, Ran, Shell, Timeout};
use file_ports::{Bytes, Files, Kind};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use store_ports::Store;

use super::super::super::super::Ground;
use super::super::super::super::crew::Program;
use super::Installing;
use crate::cli::request::InstallStep;
use crate::cli::request::ignored::Ignored;
use invariants::Ignoring;

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

impl<C: Clock, G: Shell> Installing<'_, '_, C, G> {
    pub(super) fn ran(&self, program: Program, arguments: &[&str], directory: &Path, timeout: Timeout) -> Option<String> {
        let ran = self.crew.spoke(program, arguments, directory, timeout);
        ran.as_ref().and_then(Ran::succeeded).map(|printed| AsRef::<str>::as_ref(printed).to_owned())
    }

    pub(super) fn hooked<F: Files, S: Store>(ground: &Ground<'_, F, S>, printed: &str) -> Result<(), InstallStep> {
        let settings: Printed = serde_json::from_str(printed).map_err(|_unprinted| InstallStep::Print)?;
        Self::written(ground, settings.hooks).ok_or(InstallStep::Settings)
    }

    fn written<F: Files, S: Store>(ground: &Ground<'_, F, S>, hooks: Value) -> Option<()> {
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
