mod entry;

use crate::Line;
use entry::Entry;

impl Line {
    pub(crate) fn parsed(complete: &[u8]) -> Result<Vec<Line>, serde_json::Error> {
        serde_json::Deserializer::from_slice(complete)
            .into_iter::<Entry>()
            .filter_map(|entry| entry.map(Entry::line).transpose())
            .collect()
    }
}
