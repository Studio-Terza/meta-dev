use serde::Deserialize;
use std::fmt;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rules {
    rule: Vec<Rule>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Rule {
    id: String,
    text: String,
    source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unreadable(String);

impl fmt::Display for Unreadable {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "self/rules.toml is unreadable: {}", self.0)
    }
}

impl Rules {
    pub fn parse(text: &str) -> Result<Self, Unreadable> {
        toml::from_str(text).map_err(|error| Unreadable(error.to_string()))
    }
}

impl fmt::Display for Rules {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.rule
            .iter()
            .try_for_each(|rule| writeln!(formatter, "  - {}: {}\n    source: {}", rule.id, rule.text, rule.source))
    }
}
