use std::convert::Infallible;
use std::str::FromStr;

use file_ports::Text;
use newtype::Newtype;
use toml::{Table, Value};

use super::super::Lock;
use super::Locked;
use super::Required;
use super::release::Release;
use crate::name::Name;

impl Locked {
    const PACKAGES: &'static str = "package";
    const NAME: &'static str = "name";
    const VERSION: &'static str = "version";
    const SOURCE: &'static str = "source";
    const DEPENDENCIES: &'static str = "dependencies";

    fn parsed(entry: &Value) -> Option<Locked> {
        let text = |key: &str| entry.get(key).and_then(Value::as_str);
        Some(Locked {
            name: entry.get(Self::NAME)?.as_str().map(Name::from)?,
            release: Release::new(
                text(Self::VERSION).and_then(|version| version.parse().ok()),
                text(Self::SOURCE).and_then(|origin| origin.parse().ok()),
            ),
            requires: entry
                .get(Self::DEPENDENCIES)
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .filter_map(|spelled| spelled.parse().ok())
                .collect(),
        })
    }
}

impl From<Text> for Lock {
    fn from(text: Text) -> Lock {
        let table = text.into_inner().parse::<Table>().unwrap_or_default();
        Lock(
            table
                .get(Locked::PACKAGES)
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Locked::parsed)
                .collect(),
        )
    }
}

impl FromStr for Required {
    type Err = Infallible;

    fn from_str(spelled: &str) -> Result<Required, Infallible> {
        let words: Vec<&str> = spelled.split_whitespace().collect();
        Ok(Required {
            name: words.first().copied().map(Name::from).unwrap_or_default(),
            version: words.get(1).and_then(|version| version.parse().ok()),
        })
    }
}
