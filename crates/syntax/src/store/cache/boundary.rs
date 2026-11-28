mod declared;
mod unexpanded;

use super::ports::Disk;
use declared::Column;
pub use declared::Described;
use declared::Nullable;
use serde::Deserialize;
use unexpanded::CacheFile;
pub use unexpanded::Unexpanded;

#[derive(Debug, Deserialize)]
pub struct Entry<Name> {
    describe: Describe<Name>,
}

#[derive(Debug, Deserialize)]
struct Describe<Name> {
    columns: Vec<Column<Name>>,
    parameters: Parameters,
    nullable: Vec<Nullable>,
}

#[derive(Debug, Deserialize)]
enum Parameters {
    Right(usize),
}

impl<Name: serde::de::DeserializeOwned> Entry<Name> {
    const DIRECTORY: &'static str = ".sqlx";

    pub fn found(sql: &str) -> Result<Entry<Name>, Unexpanded> {
        let file = CacheFile::from(sql);
        let named = file.to_string();
        let path = Disk::manifest()
            .ancestors()
            .find_map(|directory| Disk::held(directory.join(Self::DIRECTORY).join(&named)))
            .ok_or(Unexpanded::Uncached(file))?;
        let text = Disk::text(&path).map_err(|failed| Unexpanded::Unread { path: path.clone(), source: failed })?;
        let entry: Entry<Name> = serde_json::from_str(&text)
            .map_err(|failed| Unexpanded::Unparsed { path: path.clone(), source: failed })?;
        Ok(entry)
    }

    pub fn bound(&self, given: usize) -> Result<(), Unexpanded> {
        let Parameters::Right(wanted) = self.describe.parameters;
        (wanted == given).then_some(()).ok_or(Unexpanded::Unbound { wanted, given })
    }

    #[must_use]
    pub fn columns(self) -> Vec<Described<Name>> {
        let nullable = self.describe.nullable.into_iter().chain(std::iter::repeat(Nullable::Possible));
        self.describe.columns.into_iter().zip(nullable).map(|(column, null)| column.described(null)).collect()
    }
}
