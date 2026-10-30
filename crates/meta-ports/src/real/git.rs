use super::command::run;
use crate::{Git, PortError};
use meta_core::Origin;

#[derive(Debug, Clone, Copy)]
pub struct GitCli;

impl Git for GitCli {
    fn origin(&self) -> Result<Origin, PortError> {
        run("git", &["remote", "get-url", "origin"])?
            .parse()
            .map_err(|error| PortError::new(format!("origin: {error}")))
    }

    fn rules(&self) -> Result<String, PortError> {
        run("git", &["fetch", "--quiet", "origin", "main"])?;
        run("git", &["show", "FETCH_HEAD:self/rules.toml"])
    }
}
