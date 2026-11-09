use std::fmt;
use std::str::FromStr;

use grammar::artifact::Artifacts;
use newtype::{Display, Spelled};

use crate::ArtifactKind;

#[derive(Display, Clone, Debug, PartialEq, Eq)]
pub struct ArtifactName(String);

#[derive(Spelled, Clone, Debug, PartialEq, Eq)]
pub struct Artifact {
    kind: ArtifactKind,
    name: ArtifactName,
}

#[derive(Debug, thiserror::Error)]
#[error(
    "{0:?} is not an artifact, which is lint, hook, tool or behaviour, a colon, then its name, as in lint:no-comments"
)]
pub struct Unspecified(String);

impl FromStr for Artifact {
    type Err = Unspecified;

    fn from_str(text: &str) -> Result<Artifact, Unspecified> {
        Artifacts::spelled(text)
            .and_then(|spelled| spelled.joined(Artifact::joined))
            .ok_or_else(|| Unspecified(text.to_owned()))
    }
}

impl Artifact {
    fn joined(spelled_kind: &str, spelled_name: &str) -> Option<Artifact> {
        let name = ArtifactName(spelled_name.to_owned());
        spelled_kind.parse::<ArtifactKind>().ok().map(|kind| Artifact { kind, name })
    }
}

impl fmt::Display for Artifact {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.kind, self.name)
    }
}
