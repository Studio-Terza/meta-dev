use std::collections::BTreeSet;

use toml::de::DeTable;

use crate::line_number::LineNumber;
use crate::name::Name;
use crate::source_text::SourceText;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Owned {
    names: BTreeSet<Name>,
    line: LineNumber,
}

impl Owned {
    const METADATA: [&'static str; 3] = ["package", "metadata", "port"];
    const OWNS: &'static str = "owns";

    pub(crate) fn declared(text: &SourceText, top: &DeTable<'_>) -> Option<Owned> {
        Self::METADATA
            .iter()
            .try_fold(top, |table, wanted| {
                table.iter().find(|(key, _)| key.get_ref() == *wanted).and_then(|(_, value)| value.get_ref().as_table())
            })
            .and_then(|port| port.iter().find(|(key, _)| key.get_ref() == Self::OWNS))
            .map(|(key, value)| Owned {
                names: value
                    .get_ref()
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|owned| owned.get_ref().as_str())
                    .map(Name::from)
                    .collect(),
                line: text.line_of(key),
            })
    }
}

impl AsRef<LineNumber> for Owned {
    fn as_ref(&self) -> &LineNumber {
        &self.line
    }
}

impl AsRef<BTreeSet<Name>> for Owned {
    fn as_ref(&self) -> &BTreeSet<Name> {
        &self.names
    }
}
