use std::collections::{BTreeMap, BTreeSet};
use std::iter;

use toml::de::{DeTable, DeValue};
use workspace_metadata::{CrateName, DependencyKeys};

use super::Manifest;
use crate::line_number::LineNumber;
use crate::name::Name;
use crate::source_text::SourceText;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyLines {
    features: FeatureLines,
    owns: Option<LineNumber>,
    dependencies: DependencyLines,
    header: Option<LineNumber>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct FeatureLines(BTreeMap<Name, LineNumber>);

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct DependencyLines(BTreeMap<CrateName, BTreeSet<LineNumber>>);

impl KeyLines {
    const PORT_TABLE: [&'static str; 3] = ["package", "metadata", "port"];
    const OWNS: &'static str = "owns";
    const HEADER: &'static str = "dependencies";

    pub(super) fn read(text: &SourceText, top: &DeTable<'_>) -> KeyLines {
        KeyLines {
            features: FeatureLines::read(text, top),
            owns: Self::owns_in(text, top),
            dependencies: DependencyLines::read(text, top),
            header: top.keys().find(|key| key.get_ref() == Self::HEADER).map(|key| text.line_of(key)),
        }
    }

    #[must_use]
    pub fn features(&self) -> &BTreeMap<Name, LineNumber> {
        &self.features.0
    }

    #[must_use]
    pub fn owns(&self) -> Option<LineNumber> {
        self.owns
    }

    #[must_use]
    pub fn dependency_sites(&self, keys: &DependencyKeys<'_>) -> BTreeSet<LineNumber> {
        keys.as_ref().iter().filter_map(|key| self.dependencies.0.get(*key)).flatten().copied().collect()
    }

    #[must_use]
    pub fn dependencies_header(&self) -> LineNumber {
        self.header.unwrap_or(LineNumber::FIRST)
    }

    fn owns_in(text: &SourceText, top: &DeTable<'_>) -> Option<LineNumber> {
        let port = Self::PORT_TABLE.iter().try_fold(top, Manifest::table_in)?;
        port.keys().find(|key| key.get_ref() == Self::OWNS).map(|key| text.line_of(key))
    }
}

impl FeatureLines {
    const FEATURES: &'static str = "features";

    fn read(text: &SourceText, top: &DeTable<'_>) -> FeatureLines {
        FeatureLines(
            Manifest::entries_in(top, Self::FEATURES)
                .filter_map(DeValue::as_table)
                .flat_map(|features| features.keys())
                .map(|key| (Name::from(key.get_ref().as_ref()), text.line_of(key)))
                .collect(),
        )
    }
}

impl DependencyLines {
    fn read(text: &SourceText, top: &DeTable<'_>) -> DependencyLines {
        let platforms = Manifest::entries_in(top, Manifest::PLATFORMS)
            .filter_map(DeValue::as_table)
            .flat_map(|platforms| platforms.values())
            .filter_map(|platform| platform.get_ref().as_table());
        DependencyLines(
            iter::once(top)
                .chain(platforms)
                .flat_map(Self::lists)
                .flat_map(|listed| listed.keys())
                .filter_map(|key| Some((key.get_ref().parse::<CrateName>().ok()?, text.line_of(key))))
                .fold(BTreeMap::new(), |mut keys: BTreeMap<CrateName, BTreeSet<LineNumber>>, (key, line)| {
                    keys.entry(key).or_default().insert(line);
                    keys
                }),
        )
    }

    fn lists<'table, 'text>(table: &'table DeTable<'text>) -> impl Iterator<Item = &'table DeTable<'text>> {
        Manifest::DEPENDENCIES
            .iter()
            .flat_map(move |list| Manifest::entries_in(table, list))
            .filter_map(DeValue::as_table)
    }
}
