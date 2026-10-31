//! What git must not inherit. The build script includes this file, and so does the port, so the
//! stamp and the ancestry check ask git about the same repository.

use std::ffi::{OsStr, OsString};
use std::process::Command;

/// Leaves the variables git reads to find a repository, an index or objects (`GIT_DIR`,
/// `GIT_WORK_TREE` and the rest of `GIT_*`) unset for this command, so that `-C <repo>` alone says
/// which repository git answers for.
pub fn scrub(command: &mut Command) -> &mut Command {
    let inherited: Vec<OsString> = std::env::vars_os().map(|(name, _)| name).collect();
    let set: Vec<OsString> = command.get_envs().map(|(name, _)| name.to_owned()).collect();
    inherited.into_iter().chain(set).filter(|name| is_git(name)).for_each(|name| {
        command.env_remove(name);
    });
    command
}

/// Whether the variable's name starts with `GIT_`.
fn is_git(name: &OsStr) -> bool {
    name.to_str().is_some_and(|name| name.chars().take(4).eq("GIT_".chars()))
}
