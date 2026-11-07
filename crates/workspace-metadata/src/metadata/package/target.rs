use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Target {
    kind: Vec<Kind>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    ProcMacro,
    #[serde(other)]
    Other,
}

impl AsRef<[Kind]> for Target {
    fn as_ref(&self) -> &[Kind] {
        &self.kind
    }
}
