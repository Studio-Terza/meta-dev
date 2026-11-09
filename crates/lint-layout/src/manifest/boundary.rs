use toml::de::{DeTable, DeValue};

use super::Manifest;

impl Manifest {
    pub(super) fn table_in<'table, 'text>(
        table: &'table DeTable<'text>,
        wanted: &&str,
    ) -> Option<&'table DeTable<'text>> {
        Self::entries_in(table, wanted).find_map(DeValue::as_table)
    }

    pub(super) fn entries_in<'table, 'text>(
        table: &'table DeTable<'text>,
        wanted: &str,
    ) -> impl Iterator<Item = &'table DeValue<'text>> {
        table.iter().filter(move |(key, _)| key.get_ref() == wanted).map(|(_, value)| value.get_ref())
    }
}
