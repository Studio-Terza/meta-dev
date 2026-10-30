//! The build script: embeds the commit this binary is built from, so that only a build from a
//! commit on main writes the lead's real log. A dirty working tree, or a build outside git, embeds
//! why there is no commit instead. The binary reads these with `option_env!` (see `host.rs`).

use std::process::Command;

/// Every tracked top-level entry of the workspace, relative to this crate. The target directories
/// are left out, since every build changes them.
const SOURCES: [&str; 12] = [
    "../../.github",
    "../../.gitignore",
    "../../Cargo.lock",
    "../../Cargo.toml",
    "../../README.md",
    "../../ci",
    "../../clippy.toml",
    "../../crates",
    "../../docs",
    "../../rust-toolchain.toml",
    "../../rustfmt.toml",
    "../../self",
];

/// The git files that change on a commit, a switch or a stage.
const STATE: [&str; 3] = ["HEAD", "index", "logs/HEAD"];

fn git(args: &[&str]) -> Result<String, String> {
    let shown = format!("git {}", args.join(" "));
    let output = Command::new("git").args(args).output().map_err(|error| format!("{shown}: {error}"))?;
    match output.status.success() {
        true => String::from_utf8(output.stdout).map_err(|error| format!("{shown}: {error}")),
        false => Err(format!("{shown} failed")),
    }
}

fn embedded() -> Result<String, String> {
    let commit = git(&["log", "-1", "--format=format:%H"])?;
    match git(&["status", "--porcelain"])?.is_empty() {
        true => Ok(commit),
        false => Err(format!("a dirty working tree at {commit}")),
    }
}

fn main() {
    SOURCES.into_iter().for_each(|path| println!("cargo::rerun-if-changed={path}"));
    // `git rev-parse --git-path` ends its path with the newline that ends the directive.
    STATE.into_iter().filter_map(|file| git(&["rev-parse", "--git-path", file]).ok()).for_each(|path| {
        print!("cargo::rerun-if-changed={path}");
    });
    match embedded() {
        Ok(commit) => println!("cargo::rustc-env=META_BUILD_COMMIT={commit}"),
        Err(why) => println!("cargo::rustc-env=META_BUILD_UNVERIFIED={why}"),
    }
}
