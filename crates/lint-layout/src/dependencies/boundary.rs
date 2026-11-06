use std::iter;

use toml::de::{DeTable, DeValue};

use super::Dependencies;
use crate::dependency::Dependency;
use crate::name::Name;
use crate::source_text::SourceText;

impl Dependencies {
    const LISTS: [&'static str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];
    const PLATFORMS: &'static str = "target";
    const RENAMED: &'static str = "package";

    pub(crate) fn listed(text: &SourceText, top: &DeTable<'_>) -> Dependencies {
        let platforms = Self::entries(top, Self::PLATFORMS)
            .filter_map(DeValue::as_table)
            .flat_map(|platforms| platforms.values())
            .filter_map(|platform| platform.get_ref().as_table());
        Dependencies(
            iter::once(top)
                .chain(platforms)
                .flat_map(Self::lists)
                .flat_map(|listed| listed.iter())
                .map(|(key, value)| Dependency::new(Self::depended(key.get_ref(), value.get_ref()), text.line_of(key)))
                .collect(),
        )
    }

    fn lists<'table, 'text>(table: &'table DeTable<'text>) -> impl Iterator<Item = &'table DeTable<'text>> {
        Self::LISTS.iter().flat_map(move |list| Self::entries(table, list)).filter_map(DeValue::as_table)
    }

    fn entries<'table, 'text>(
        table: &'table DeTable<'text>,
        wanted: &str,
    ) -> impl Iterator<Item = &'table DeValue<'text>> {
        table.iter().filter(move |(key, _)| key.get_ref() == wanted).map(|(_, value)| value.get_ref())
    }

    fn depended(key: &str, value: &DeValue<'_>) -> Name {
        value
            .as_table()
            .into_iter()
            .flat_map(|table| Self::entries(table, Self::RENAMED))
            .find_map(DeValue::as_str)
            .map_or_else(|| Name::from(key), Name::from)
    }
}
