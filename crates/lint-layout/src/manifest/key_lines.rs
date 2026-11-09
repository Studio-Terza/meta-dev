use std::collections::BTreeMap;

use toml::de::DeTable;

use super::Manifest;
use crate::line_number::LineNumber;
use crate::name::Name;
use crate::source_text::SourceText;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyLines {
    features: BTreeMap<Name, LineNumber>,
    owns: Option<LineNumber>,
}

impl KeyLines {
    const FEATURES: &'static str = "features";
    const PORT_TABLE: [&'static str; 3] = ["package", "metadata", "port"];
    const OWNS: &'static str = "owns";

    pub(super) fn read(text: &SourceText, top: &DeTable<'_>) -> KeyLines {
        KeyLines { features: Self::features_in(text, top), owns: Self::owns_in(text, top) }
    }

    #[must_use]
    pub fn features(&self) -> &BTreeMap<Name, LineNumber> {
        &self.features
    }

    #[must_use]
    pub fn owns(&self) -> Option<LineNumber> {
        self.owns
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
}
