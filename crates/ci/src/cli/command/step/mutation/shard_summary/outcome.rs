use std::path::PathBuf;

use strum::{Display, VariantArray};

#[derive(Display, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum Outcome {
    Caught,
    Missed,
    Timeout,
    Unviable,
}

impl Outcome {
    pub(crate) const LISTED: [Outcome; 2] = [Outcome::Missed, Outcome::Timeout];

    const EXTENSION: &'static str = ".txt";

    pub(crate) fn file_name(self) -> PathBuf {
        PathBuf::from([self.to_string().as_str(), Self::EXTENSION].concat())
    }
}
