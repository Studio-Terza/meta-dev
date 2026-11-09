use toml::de::DeTable;

use super::Manifest;
use crate::line_number::LineNumber;
use crate::source_text::SourceText;

impl Manifest {
    pub(super) fn table_in<'table, 'text>(
        table: &'table DeTable<'text>,
        wanted: &&str,
    ) -> Option<&'table DeTable<'text>> {
        table.iter().find(|(key, _)| key.get_ref() == *wanted).and_then(|(_, value)| value.get_ref().as_table())
    }

    pub(super) fn key_in(text: &SourceText, top: &DeTable<'_>, table: &[&str], wanted: &str) -> Option<LineNumber> {
        let found = table.iter().try_fold(top, Manifest::table_in)?;
        found.keys().find(|key| key.get_ref() == wanted).map(|key| text.line_of(key))
    }
}
