use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    Library,
    ProcMacro,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum TargetKind {
    Lib,
    Rlib,
    Dylib,
    ProcMacro,
    #[serde(other)]
    Other,
}

impl TargetKind {
    fn form(self) -> Option<Form> {
        match self {
            TargetKind::Lib | TargetKind::Rlib | TargetKind::Dylib => Some(Form::Library),
            TargetKind::ProcMacro => Some(Form::ProcMacro),
            TargetKind::Other => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Target {
    kind: Vec<TargetKind>,
    src_path: PathBuf,
}

impl Target {
    pub fn form(&self) -> Option<Form> {
        self.kind.iter().find_map(|kind| kind.form())
    }

    pub fn root(&self) -> &Path {
        &self.src_path
    }
}
