use serde::Deserialize;

#[derive(Deserialize)]
#[serde(try_from = "String")]
pub struct Title(String);

#[derive(Deserialize)]
pub struct Entry {
    title: Title,
}
