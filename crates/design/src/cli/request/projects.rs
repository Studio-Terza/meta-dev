mod slug;

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use console_ports::Environment;

use crate::cli::failure::Failure;
use slug::Slug;

#[derive(Debug)]
pub struct Projects;

impl Projects {
    pub fn directory(environment: &impl Environment, workspace: Option<&Path>) -> Result<PathBuf, Failure> {
        let home = environment.variable(OsStr::new("HOME")).ok_or(Failure::Unset(OsString::from("HOME")))?;
        let current = environment.variable(OsStr::new("PWD")).map(PathBuf::from);
        let chosen = workspace.map(Path::to_path_buf).or(current).ok_or(Failure::Unset(OsString::from("PWD")))?;
        let slug = Slug::try_from(chosen.as_path())?;
        Ok(PathBuf::from(home).join(".claude").join("projects").join(PathBuf::from(slug)))
    }
}
