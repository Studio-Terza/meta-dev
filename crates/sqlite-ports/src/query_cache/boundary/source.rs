use std::fs;
use std::path::{Path, PathBuf};

use proc_macro2::TokenStream;
use syn::parse::{ParseStream, Parser};
use syn::visit::Visit;
use syn::{LitStr, Macro, Token, Type};

use super::Uncached;

#[derive(Debug)]
pub(super) struct Sources(Vec<PathBuf>);

#[derive(Debug, Default)]
struct Queries(Vec<String>);

impl Sources {
    const CRATES: &'static str = "crates";
    const RUST: &'static str = "rs";
    const FIXTURES: &'static str = "fixtures";

    pub(super) fn under(root: &Path) -> Result<Sources, Uncached> {
        Self::walked(&root.join(Self::CRATES)).map(Sources)
    }

    pub(super) fn queries(&self) -> Result<Vec<String>, Uncached> {
        self.0
            .iter()
            .try_fold(Queries::default(), |mut found, path| {
                let text =
                    fs::read_to_string(path).map_err(|failed| Uncached::Io { path: path.clone(), source: failed })?;
                let _visited = syn::parse_file(&text).map(|file| found.visit_file(&file));
                Ok(found)
            })
            .map(|found| found.0)
    }

    fn walked(directory: &Path) -> Result<Vec<PathBuf>, Uncached> {
        let unlisted = |failed| Uncached::Io { path: directory.to_owned(), source: failed };
        fs::read_dir(directory)
            .map_err(unlisted)?
            .map(|entry| entry.map_err(unlisted).and_then(|found| Self::expanded(&found)))
            .collect::<Result<Vec<Vec<PathBuf>>, Uncached>>()
            .map(|nested| nested.into_iter().flatten().collect())
    }

    fn expanded(entry: &fs::DirEntry) -> Result<Vec<PathBuf>, Uncached> {
        let path = entry.path();
        let fixture = path.file_name().filter(|name| *name == Self::FIXTURES).map(|_fixture| Ok(Vec::new()));
        fixture.or_else(|| path.is_dir().then(|| Self::walked(&path))).unwrap_or_else(|| {
            Ok(path
                .extension()
                .filter(|extension| *extension == Self::RUST)
                .map(|_rust| path.clone())
                .into_iter()
                .collect())
        })
    }
}

impl Queries {
    const QUERY: &'static str = "query";
    const PORT: &'static str = "store_ports";

    fn literal(input: ParseStream<'_>) -> syn::Result<LitStr> {
        let literal = input.fork().parse::<LitStr>().map_or_else(
            |_| {
                input.parse::<Type>()?;
                input.parse::<Token![,]>()?;
                input.parse::<LitStr>()
            },
            |_| input.parse::<LitStr>(),
        )?;
        input.parse::<TokenStream>()?;
        Ok(literal)
    }
}

impl<'ast> Visit<'ast> for Queries {
    fn visit_macro(&mut self, invocation: &'ast Macro) {
        let named = invocation.path.segments.iter().map(|segment| segment.ident.to_string()).collect::<Vec<String>>();
        named
            .first()
            .filter(|port| *port == Self::PORT)
            .and(named.last())
            .filter(|last| *last == Self::QUERY)
            .and_then(|_query| Queries::literal.parse2(invocation.tokens.clone()).ok())
            .into_iter()
            .for_each(|literal| self.0.push(literal.value()));
        syn::visit::visit_macro(self, invocation);
    }
}
