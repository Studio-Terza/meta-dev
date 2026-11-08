use std::fmt::Display;

use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Listed(String);

impl<Item: Display> FromIterator<Item> for Listed {
    fn from_iter<Items: IntoIterator<Item = Item>>(items: Items) -> Listed {
        let spelled: Vec<String> = items.into_iter().map(|item| item.to_string()).collect();
        Listed(serde_json::Value::from(spelled).to_string())
    }
}
