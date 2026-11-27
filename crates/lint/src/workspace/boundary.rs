use toml::de::{DeTable, DeValue};

use super::{Feature, Workspace};
use crate::terms::{Line, Name};

impl Workspace {
    const NAME: &'static str = "name";
    const PATH: &'static str = "path";
    const FEATURES: &'static str = "features";

    pub(super) fn named(value: &DeValue<'_>) -> Option<Name> {
        Workspace::string(value, Workspace::NAME).map(Name::spelled)
    }

    pub(super) fn pathed<'value>(value: &'value DeValue<'_>) -> Option<&'value str> {
        Workspace::string(value, Workspace::PATH)
    }

    fn string<'value>(value: &'value DeValue<'_>, key: &str) -> Option<&'value str> {
        value.as_table().and_then(|table| table.get(key)).and_then(|found| found.get_ref().as_str())
    }

    pub(super) fn features(text: &str, table: &DeTable<'_>) -> Vec<Feature> {
        let declared = table.get(Workspace::FEATURES).and_then(|features| features.get_ref().as_table());
        declared.map_or_default(|features| {
            features
                .iter()
                .map(|(key, _)| Feature {
                    name: Name::spelled(key.get_ref()),
                    line: Line::numbered(
                        text.get(..key.span().start).map_or(0, |before| before.matches('\n').count()) + 1,
                    ),
                })
                .collect()
        })
    }
}
