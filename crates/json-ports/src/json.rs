use crate::{Decode, Encode};

#[derive(Debug)]
pub struct Json;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct JsonFailure(#[from] serde_json::Error);

impl Json {
    pub fn read<Read: Decode>(text: &str) -> Result<Read, JsonFailure> {
        Ok(serde_json::from_str(text)?)
    }

    #[must_use]
    pub fn first<Read: Decode>(text: &str) -> Option<Result<Read, JsonFailure>> {
        serde_json::Deserializer::from_str(text).into_iter().next().map(|read| read.map_err(JsonFailure::from))
    }

    pub fn written<Written: Encode + ?Sized>(value: &Written) -> Result<String, JsonFailure> {
        Ok(serde_json::to_string(value)?)
    }

    pub fn pretty<Written: Encode + ?Sized>(value: &Written) -> Result<String, JsonFailure> {
        Ok(serde_json::to_string_pretty(value)?)
    }
}
