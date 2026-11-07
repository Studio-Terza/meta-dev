use serde::Serialize;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct Title(String);

impl From<String> for Title {
    fn from(text: String) -> Self {
        Self(text)
    }
}
