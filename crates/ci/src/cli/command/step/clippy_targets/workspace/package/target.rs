use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct Target {
    kind: Vec<String>,
}

impl Target {
    pub(super) fn kinds(&self) -> impl Iterator<Item = &str> {
        self.kind.iter().map(String::as_str)
    }
}
