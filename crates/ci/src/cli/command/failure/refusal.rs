use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub struct Refusal(String);

impl From<Vec<u8>> for Refusal {
    fn from(bytes: Vec<u8>) -> Refusal {
        Refusal(String::from_utf8(bytes).unwrap_or_default())
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
