use std::fmt;

use file_ports::Text;

#[derive(Debug, PartialEq, Eq)]
pub struct Refusal(String);

impl From<Vec<u8>> for Refusal {
    fn from(bytes: Vec<u8>) -> Self {
        Self(String::from_utf8(bytes).unwrap_or_default())
    }
}

impl From<Text> for Refusal {
    fn from(text: Text) -> Self {
        Self(text.as_ref().to_owned())
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
