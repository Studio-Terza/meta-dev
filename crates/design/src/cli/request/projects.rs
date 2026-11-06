mod slug;

use std::ffi::{OsStr, OsString};
use std::path::{Component, Path, PathBuf};

use console_ports::Environment;

use crate::cli::failure::Failure;
use slug::Slug;

#[derive(Debug)]
pub struct Projects;

impl Projects {
    pub fn directory(environment: &impl Environment, workspace: Option<&Path>) -> Result<PathBuf, Failure> {
        let home = environment.variable(OsStr::new("HOME")).ok_or(Failure::Unset(OsString::from("HOME")))?;
        let current = environment.variable(OsStr::new("PWD")).map(PathBuf::from);
        let chosen = Self::chosen(current, workspace).ok_or(Failure::Unset(OsString::from("PWD")))?;
        let slug = Slug::try_from(Self::resolved(&chosen).as_path())?;
        Ok(PathBuf::from(home).join(".claude").join("projects").join(PathBuf::from(slug)))
    }

    fn chosen(current: Option<PathBuf>, workspace: Option<&Path>) -> Option<PathBuf> {
        match workspace {
            Some(workspace) => Some(current.unwrap_or_default().join(workspace)),
            None => current,
        }
    }

    fn resolved(path: &Path) -> PathBuf {
        path.components().fold(PathBuf::new(), Self::stepped)
    }

    fn stepped(resolved: PathBuf, component: Component<'_>) -> PathBuf {
        match component {
            Component::ParentDir => resolved.parent().map(Path::to_path_buf).unwrap_or(resolved),
            Component::CurDir => resolved,
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => resolved.join(component),
        }
    }
}
