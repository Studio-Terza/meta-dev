use std::collections::{BTreeMap, BTreeSet};
use std::iter;

use text_lines::OneLine;
use toml::de::{DeTable, DeValue};
use workspace_metadata::{CrateName, DependencyKeys};

use super::Manifest;
use crate::line_number::LineNumber;
use crate::name::Name;
use crate::source_text::SourceText;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyLines {
    features: BTreeMap<Name, LineNumber>,
    owns: Option<LineNumber>,
    dependencies: BTreeMap<CrateName, BTreeSet<LineNumber>>,
    header: Option<LineNumber>,
}

impl KeyLines {
    const FEATURES: &'static str = "features";
    const PORT_TABLE: [&'static str; 3] = ["package", "metadata", "port"];
    const OWNS: &'static str = "owns";
    const HEADER: &'static str = "dependencies";

    pub(super) fn read(text: &SourceText, top: &DeTable<'_>) -> KeyLines {
        KeyLines {
            features: Self::features_in(text, top),
            owns: Self::owns_in(text, top),
            dependencies: Self::dependencies_in(text, top),
            header: top.keys().find(|key| key.get_ref() == Self::HEADER).map(|key| text.line_of(key)),
        }
    }

    #[must_use]
    pub fn features(&self) -> &BTreeMap<Name, LineNumber> {
        &self.features
    }

    #[must_use]
    pub fn owns(&self) -> Option<LineNumber> {
        self.owns
    }

    #[must_use]
    pub fn dependency_sites(&self, keys: &DependencyKeys<'_>) -> BTreeSet<LineNumber> {
        keys.as_ref().iter().filter_map(|key| self.dependencies.get(*key)).flatten().copied().collect()
    }

    #[must_use]
    pub fn dependencies_header(&self) -> LineNumber {
        self.header.unwrap_or(LineNumber::FIRST)
    }

    fn features_in(text: &SourceText, top: &DeTable<'_>) -> BTreeMap<Name, LineNumber> {
        top.iter()
            .filter(|(key, _)| key.get_ref() == Self::FEATURES)
            .filter_map(|(_, value)| value.get_ref().as_table())
            .flat_map(|features| features.keys())
            .map(|key| (Name::from(key.get_ref().as_ref()), text.line_of(key)))
            .collect()
    }

    fn owns_in(text: &SourceText, top: &DeTable<'_>) -> Option<LineNumber> {
        let port = Self::PORT_TABLE.iter().try_fold(top, Manifest::table_in)?;
        port.keys().find(|key| key.get_ref() == Self::OWNS).map(|key| text.line_of(key))
    }

    fn dependencies_in(text: &SourceText, top: &DeTable<'_>) -> BTreeMap<CrateName, BTreeSet<LineNumber>> {
        let platforms = Self::entries(top, Manifest::PLATFORMS)
            .filter_map(DeValue::as_table)
            .flat_map(|platforms| platforms.values())
            .filter_map(|platform| platform.get_ref().as_table());
        iter::once(top)
            .chain(platforms)
            .flat_map(Self::lists)
            .flat_map(|listed| listed.keys())
            .filter_map(|key| {
                let named = OneLine::try_new(key.get_ref().to_string()).ok().map(CrateName::new)?;
                Some((named, text.line_of(key)))
            })
            .fold(BTreeMap::new(), |mut keys: BTreeMap<CrateName, BTreeSet<LineNumber>>, (key, line)| {
                keys.entry(key).or_default().insert(line);
                keys
            })
    }

    fn lists<'table, 'text>(table: &'table DeTable<'text>) -> impl Iterator<Item = &'table DeTable<'text>> {
        Manifest::DEPENDENCIES.iter().flat_map(move |list| Self::entries(table, list)).filter_map(DeValue::as_table)
    }

    fn entries<'table, 'text>(
        table: &'table DeTable<'text>,
        wanted: &str,
    ) -> impl Iterator<Item = &'table DeValue<'text>> {
        table.iter().filter(move |(key, _)| key.get_ref() == wanted).map(|(_, value)| value.get_ref())
    }
}
