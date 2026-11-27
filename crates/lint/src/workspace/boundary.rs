use toml::de::{DeTable, DeValue};

use super::Feature;
use crate::finding::Line;
use crate::name::Name;

#[derive(Debug)]
pub(super) struct Toml;

impl Toml {
    pub(super) const NAME: &'static str = "name";
    pub(super) const PATH: &'static str = "path";
    const FEATURES: &'static str = "features";

    pub(super) fn string<'value>(value: &'value DeValue<'_>, key: &str) -> Option<&'value str> {
        value.as_table().and_then(|table| table.get(key)).and_then(|found| found.get_ref().as_str())
    }

    pub(super) fn features(text: &str, table: &DeTable<'_>) -> Vec<Feature> {
        let declared = table.get(Toml::FEATURES).and_then(|features| features.get_ref().as_table());
        declared.map_or_default(|features| {
            features
                .iter()
                .map(|(key, _)| Feature {
                    name: Name::spelled(key.get_ref()),
                    line: Line::new(text.get(..key.span().start).map_or(0, |before| before.matches('\n').count()) + 1),
                })
                .collect()
        })
    }
}
