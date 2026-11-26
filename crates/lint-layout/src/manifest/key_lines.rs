use std::collections::BTreeMap;

use toml::de::DeTable;

use crate::line_number::LineNumber;
use crate::name::Name;
use crate::source_text::SourceText;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyLines {
    features: BTreeMap<Name, LineNumber>,
}

impl KeyLines {
    const FEATURES: &'static str = "features";

    pub(super) fn read(text: &SourceText, top: &DeTable<'_>) -> KeyLines {
        KeyLines { features: Self::features_in(text, top) }
    }

    #[must_use]
    pub fn features(&self) -> &BTreeMap<Name, LineNumber> {
        &self.features
    }

    fn features_in(text: &SourceText, top: &DeTable<'_>) -> BTreeMap<Name, LineNumber> {
        top.iter()
            .filter(|(key, _)| key.get_ref() == Self::FEATURES)
            .filter_map(|(_, value)| value.get_ref().as_table())
            .flat_map(|features| features.keys())
            .map(|key| (Name::from(key.get_ref().as_ref()), text.line_of(key)))
            .collect()
    }
}
