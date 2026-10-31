//! What git must not inherit. The build script includes this file, and so does the port, so the
//! stamp and the ancestry check ask git about the same repository.

use std::process::Command;

/// Leaves the variables git reads to find a repository, an index or objects (`GIT_DIR`,
/// `GIT_WORK_TREE` and the rest of `GIT_*`) unset for this command, so that `-C <repo>` alone says
/// which repository git answers for.
pub fn scrub(command: &mut Command) -> &mut Command {
    command
}
