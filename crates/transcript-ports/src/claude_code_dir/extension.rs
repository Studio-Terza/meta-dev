use std::str::FromStr;

#[derive(Debug, thiserror::Error)]
#[error("not a transcript file extension")]
pub struct Foreign;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Extension {
    Jsonl,
}

impl FromStr for Extension {
    type Err = Foreign;

    fn from_str(name: &str) -> Result<Extension, Foreign> {
        Self::NAMED.iter().find(|(known, _)| *known == name).map(|(_, extension)| *extension).ok_or(Foreign)
    }
}

impl Extension {
    const NAMED: [(&'static str, Extension); 1] = [("jsonl", Extension::Jsonl)];
}
