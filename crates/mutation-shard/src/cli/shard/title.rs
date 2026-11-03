use std::fmt::Write;

use serde::Serialize;

use super::named::Named;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct Title(String);

impl From<&Named> for Title {
    fn from(named: &Named) -> Title {
        let mut title = String::new();
        write!(title, "{named}").ok();
        Title(title)
    }
}
