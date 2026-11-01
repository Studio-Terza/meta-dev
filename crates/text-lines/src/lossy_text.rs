use std::fmt;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LossyText(String);

impl LossyText {
    const REPLACEMENT: &'static str = "\u{FFFD}";
}

impl From<&[u8]> for LossyText {
    fn from(bytes: &[u8]) -> LossyText {
        LossyText(
            bytes
                .utf8_chunks()
                .flat_map(|chunk| [chunk.valid(), if chunk.invalid().is_empty() { "" } else { Self::REPLACEMENT }])
                .collect(),
        )
    }
}

impl AsRef<str> for LossyText {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for LossyText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
