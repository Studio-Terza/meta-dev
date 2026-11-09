mod declared;
mod unexpanded;

use std::path::PathBuf;

use proc_macro2::TokenStream;
use quote::quote;
use serde::Deserialize;
use syn::LitStr;

use crate::ports::Disk;
pub use declared::Typed;
use declared::{Column, Nullable};
use unexpanded::CacheFile;
pub use unexpanded::Unexpanded;

#[derive(Debug, Deserialize)]
pub struct Entry {
    #[serde(skip)]
    path: PathBuf,
    describe: Describe,
}

#[derive(Debug, Deserialize)]
struct Describe {
    columns: Vec<Column>,
    parameters: Parameters,
    nullable: Vec<Nullable>,
}

#[derive(Debug, Deserialize)]
enum Parameters {
    Right(usize),
}

impl Entry {
    const DIRECTORY: &'static str = ".sqlx";

    pub fn found(source: &LitStr) -> Result<Entry, Unexpanded> {
        let file = CacheFile::from(source);
        let named = file.to_string();
        let path = Disk::manifest()
            .ancestors()
            .find_map(|directory| Disk::held(directory.join(Self::DIRECTORY).join(&named)))
            .ok_or(Unexpanded::Uncached(file))?;
        let text = Disk::text(&path).map_err(|failed| Unexpanded::Unread { path: path.clone(), source: failed })?;
        let entry: Entry = serde_json::from_str(&text)
            .map_err(|failed| Unexpanded::Unparsed { path: path.clone(), source: failed })?;
        Ok(Entry { path, ..entry })
    }

    pub fn tracked(&self) -> TokenStream {
        let path = self.path.to_string_lossy();
        quote!(
            const _: &[u8] = include_bytes!(#path);
        )
    }

    pub fn bound(&self, given: usize) -> Result<(), Unexpanded> {
        let Parameters::Right(wanted) = self.describe.parameters;
        (wanted == given).then_some(()).ok_or(Unexpanded::Unbound { wanted, given })
    }

    pub fn columns(self) -> Result<Vec<Typed>, Unexpanded> {
        let nullable = self.describe.nullable.into_iter().chain(std::iter::repeat(Nullable::Possible));
        self.describe.columns.into_iter().zip(nullable).map(|(column, null)| column.typed(null)).collect()
    }
}
