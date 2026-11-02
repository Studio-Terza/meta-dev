use std::fmt;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(try_from = "String")]
pub struct OneLine(String);

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("a line of text holds no line break or other control character")]
pub struct NotOneLine;

impl TryFrom<String> for OneLine {
    type Error = NotOneLine;

    fn try_from(text: String) -> Result<OneLine, NotOneLine> {
        if text.chars().any(char::is_control) { Err(NotOneLine) } else { Ok(OneLine(text)) }
    }
}

impl AsRef<str> for OneLine {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for OneLine {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
