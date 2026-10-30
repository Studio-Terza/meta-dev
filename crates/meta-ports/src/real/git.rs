use super::command::run;
use crate::{Git, PortError};
use meta_core::{Origin, SelfFile};

#[derive(Debug, Clone, Copy)]
pub struct GitCli;

impl Git for GitCli {
    fn origin(&self) -> Result<Origin, PortError> {
        run("git", &["remote", "get-url", "origin"])?
            .parse()
            .map_err(|error| PortError::new(format!("origin: {error}")))
    }

    fn read(&self, file: SelfFile) -> Result<String, PortError> {
        run("git", &["fetch", "--quiet", "origin", "main"])?;
        run("git", &["show", &format!("FETCH_HEAD:{}", file.path())])
    }
}
