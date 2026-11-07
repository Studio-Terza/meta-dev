use toml::de::DeTable;

use super::Manifest;

impl Manifest {
    pub(super) fn table_in<'table, 'text>(
        table: &'table DeTable<'text>,
        wanted: &&str,
    ) -> Option<&'table DeTable<'text>> {
        table.iter().find(|(key, _)| key.get_ref() == *wanted).and_then(|(_, value)| value.get_ref().as_table())
    }
}
