mod declared;
mod unexpanded;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::ports::Disk;
pub use declared::Described;
use declared::{Column, Nullable};
pub use unexpanded::{CacheFile, Unexpanded};

#[derive(Debug, Deserialize, Serialize)]
pub struct Entry<Name> {
    query: String,
    describe: Describe<Name>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Describe<Name> {
    columns: Vec<Column<Name>>,
    parameters: Parameters,
    nullable: Vec<Nullable>,
}

#[derive(Debug, Deserialize, Serialize)]
enum Parameters {
    Right(usize),
}

impl<Name> Entry<Name> {
    const DIRECTORY: &'static str = ".sqlx";

    #[must_use]
    pub fn described(query: &str, described: Vec<Described<Name>>, parameters: usize) -> Entry<Name> {
        let nullable = described.iter().map(Described::nullable).collect();
        let columns = described.into_iter().map(Described::into_column).collect();
        Entry {
            query: query.to_owned(),
            describe: Describe { columns, parameters: Parameters::Right(parameters), nullable },
        }
    }

    #[must_use]
    pub fn file(&self) -> CacheFile {
        CacheFile::from(self.query.as_str())
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

impl<Name: Serialize> Entry<Name> {
    pub fn written(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self).map(|text| format!("{text}\n"))
    }
}

impl<Name: DeserializeOwned> Entry<Name> {
    pub fn found(sql: &str) -> Result<Entry<Name>, Unexpanded> {
        let file = CacheFile::from(sql);
        let named = file.to_string();
        let path = Disk::manifest()
            .ancestors()
            .find_map(|directory| Disk::held(directory.join(Entry::<Name>::DIRECTORY).join(&named)))
            .ok_or(Unexpanded::Uncached(file))?;
        let text = Disk::text(&path).map_err(|failed| Unexpanded::Unread { path: path.clone(), source: failed })?;
        serde_json::from_str(&text).map_err(|failed| Unexpanded::Unparsed { path, source: failed })
    }
}
