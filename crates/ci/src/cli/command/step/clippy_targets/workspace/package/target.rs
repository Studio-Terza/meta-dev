use std::collections::BTreeSet;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct Target {
    name: String,
    kind: Vec<String>,
    #[serde(default, rename = "required-features")]
    required_features: Vec<String>,
    test: Option<bool>,
}

impl Target {
    const INTEGRATION: &'static str = "test";

    pub(super) fn name(&self) -> &str {
        &self.name
    }

    pub(super) fn buildable_with(&self, enabled: &BTreeSet<&str>) -> bool {
        self.kind.iter().any(|kind| kind == Self::INTEGRATION)
            && self.test.unwrap_or(true)
            && self.required_features.iter().all(|feature| enabled.contains(feature.as_str()))
    }
}
