mod entry;

use super::Record;
use entry::Entry;

impl Record {
    pub(crate) fn parsed(complete: &[u8]) -> Result<Vec<Record>, serde_json::Error> {
        serde_json::Deserializer::from_slice(complete)
            .into_iter::<Entry>()
            .filter_map(|entry| entry.map(Entry::record).transpose())
            .collect()
    }
}
