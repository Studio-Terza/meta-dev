use std::collections::{BTreeMap, BTreeSet};
use std::iter;

use text_lines::OneLine;
use toml::Spanned;
use toml::de::{DeTable, DeValue};
use workspace_metadata::CrateName;

use super::DependencySites;
use crate::line_number::LineNumber;
use crate::source_text::SourceText;

impl DependencySites {
    const LISTS: [&'static str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];
    const PLATFORMS: &'static str = "target";
    const HEADER: &'static str = "dependencies";

    pub(crate) fn listed(text: &SourceText, top: &DeTable<'_>) -> DependencySites {
        let platforms = Self::entries(top, Self::PLATFORMS)
            .filter_map(DeValue::as_table)
            .flat_map(|platforms| platforms.values())
            .filter_map(|platform| platform.get_ref().as_table());
        let keys = iter::once(top)
            .chain(platforms)
            .flat_map(Self::lists)
            .flat_map(|listed| listed.keys())
            .filter_map(|key| {
                Some((CrateName::new(OneLine::try_new(key.get_ref().clone().into_owned()).ok()?), text.line_of(key)))
            })
            .fold(BTreeMap::new(), |mut keys: BTreeMap<CrateName, BTreeSet<LineNumber>>, (key, line)| {
                keys.entry(key).or_default().insert(line);
                keys
            });
        let header = top.keys().find(|key| key.get_ref() == Self::HEADER).map(|key| text.line_of(key));
        DependencySites { keys, header }
    }

    fn lists<'table, 'text>(table: &'table DeTable<'text>) -> impl Iterator<Item = &'table DeTable<'text>> {
        Self::LISTS.iter().flat_map(move |list| Self::entries(table, list)).filter_map(DeValue::as_table)
    }

    fn entries<'table, 'text>(
        table: &'table DeTable<'text>,
        wanted: &str,
    ) -> impl Iterator<Item = &'table DeValue<'text>> {
        table.get(wanted).map(Spanned::get_ref).into_iter()
    }
}
