//! The build script: embeds the commit this binary is built from, so that only a build from a
//! commit on main writes the lead's real log. Edits to tracked files, or a build outside git, embed
//! why there is no clean commit instead. The binary reads these with `option_env!` (see `host.rs`);
//! the logic is in `stamp.rs`, where it is tested.

#[path = "git_env.rs"]
mod git_env;
#[path = "stamp.rs"]
mod stamp;

fn main() {
    print!("{}", stamp::directives(std::path::Path::new(".")));
}
