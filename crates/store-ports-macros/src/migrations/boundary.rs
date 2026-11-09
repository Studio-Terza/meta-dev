use std::collections::BTreeMap;
use std::io;
use std::path::PathBuf;

use grammar_migration::MigrationFile;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::LitStr;

use crate::ports::Disk;

#[derive(Debug)]
pub struct Directory(PathBuf);

#[derive(Debug)]
pub struct Found {
    file: MigrationFile,
    path: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum Unmigrated {
    #[error("cannot list the migrations in {}: {source}", path.display())]
    Unlisted { path: PathBuf, source: io::Error },
    #[error("{} is not named <version>_<description>.sql", path.display())]
    Unnamed { path: PathBuf },
    #[error("{} repeats migration version {version}", path.display())]
    Twice { path: PathBuf, version: i64 },
}

impl Directory {
    const DEFAULT: &'static str = "migrations";

    pub fn named(relative: Option<&LitStr>) -> Directory {
        Directory(Disk::manifest().join(relative.map_or_else(|| Self::DEFAULT.to_owned(), LitStr::value)))
    }

    pub fn migrations(&self) -> Result<Vec<Found>, Unmigrated> {
        let paths = Disk::listed(&self.0).map_err(|source| Unmigrated::Unlisted { path: self.0.clone(), source })?;
        let listed = paths.into_iter().map(Found::read).try_fold(
            BTreeMap::new(),
            |mut listed: BTreeMap<i64, Found>, read| {
                let found = read?;
                let version = found.file.version();
                listed
                    .insert(version, found)
                    .map_or(Ok(listed), |twice| Err(Unmigrated::Twice { path: twice.path, version }))
            },
        )?;
        Ok(listed.into_values().collect())
    }
}

impl Found {
    fn read(path: PathBuf) -> Result<Found, Unmigrated> {
        let named = path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
        let file = named.parse::<MigrationFile>().map_err(|_unspelled| Unmigrated::Unnamed { path: path.clone() })?;
        Ok(Found { file, path })
    }
}

impl ToTokens for Found {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let version = self.file.version();
        let description = self.file.description();
        let path = self.path.to_string_lossy();
        let new = quote!(::store_ports::Migration::new);
        tokens.extend(quote!(#new(#version, #description, include_str!(#path))));
    }
}
