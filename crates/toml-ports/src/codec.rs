use json_ports::{Decode, Encode};

#[derive(Debug)]
pub struct Toml;

#[derive(Debug, thiserror::Error)]
pub enum TomlFailure {
    #[error(transparent)]
    Read(#[from] toml::de::Error),
    #[error(transparent)]
    Write(#[from] toml::ser::Error),
}

impl Toml {
    pub fn read<Read: Decode>(text: &str) -> Result<Read, TomlFailure> {
        Ok(toml::from_str(text)?)
    }

    pub fn written<Written: Encode + ?Sized>(value: &Written) -> Result<String, TomlFailure> {
        Ok(toml::to_string(value)?)
    }
}
