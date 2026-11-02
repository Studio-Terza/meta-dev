use serde::Deserialize;

#[derive(Deserialize)]
pub struct Entry {
    title: String,
}
