use std::str::FromStr;

use super::Workspace;
use crate::cli::command::failure::Failure;

impl FromStr for Workspace {
    type Err = Failure<'static>;

    fn from_str(text: &str) -> Result<Workspace, Failure<'static>> {
        serde_json::from_str(text).map_err(Failure::Metadata)
    }
}
