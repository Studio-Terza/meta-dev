use std::path::{Path, PathBuf};

use file_ports::{Files, Kind};
use query_cache::{Invoked, Recorded};
use syn::visit::Visit;
use syn::{ItemStruct, LitStr, Macro};

use crate::uncached::Uncached;

#[derive(Debug, Default)]
pub(crate) struct Sources {
    found: Vec<syn::Result<Vec<LitStr>>>,
}

impl Sources {
    const PORT: &'static str = "store_ports";
    const QUERY: &'static str = "query";
    const FIXTURES: &'static str = "fixtures";
    const RUST: &'static str = "rs";

    pub(crate) fn queries(files: &impl Files, crates: &Path) -> Result<Vec<LitStr>, Uncached> {
        Sources::rust(files, crates)?.iter().try_fold(Vec::new(), |mut queries, path| {
            let text = files.read_text(path).map_err(|source| Uncached::Unread { path: path.clone(), source })?;
            let parsed =
                syn::parse_file(text.as_ref()).map_err(|source| Uncached::Unparsed { path: path.clone(), source })?;
            let mut sources = Sources::default();
            sources.visit_file(&parsed);
            let found = sources.found.into_iter().collect::<syn::Result<Vec<Vec<LitStr>>>>();
            queries.extend(
                found.map_err(|source| Uncached::Unparsed { path: path.clone(), source })?.into_iter().flatten(),
            );
            Ok(queries)
        })
    }

    fn rust(files: &impl Files, directory: &Path) -> Result<Vec<PathBuf>, Uncached> {
        let entries =
            files.entries(directory).map_err(|source| Uncached::Unread { path: directory.to_path_buf(), source })?;
        let nested = entries
            .into_iter()
            .filter(|path| path.file_name().is_none_or(|name| name != Sources::FIXTURES))
            .map(|path| match files.kind(&path) {
                Kind::Directory => Sources::rust(files, &path),
                Kind::File | Kind::Link | Kind::Missing => Ok(Some(path)
                    .filter(|file| file.extension().is_some_and(|extension| extension == Sources::RUST))
                    .into_iter()
                    .collect()),
            })
            .collect::<Result<Vec<Vec<PathBuf>>, Uncached>>()?;
        Ok(nested.into_iter().flatten().collect())
    }
}

impl<'ast> Visit<'ast> for Sources {
    fn visit_macro(&mut self, invocation: &'ast Macro) {
        let segments = &invocation.path.segments;
        let ported = segments.first().filter(|port| port.ident == Sources::PORT);
        let query = ported.and(segments.last()).filter(|last| last.ident == Sources::QUERY);
        self.found.extend(
            query.map(|_query| invocation.parse_body::<Invoked>().map(|invoked| vec![invoked.source().clone()])),
        );
        syn::visit::visit_macro(self, invocation);
    }

    fn visit_item_struct(&mut self, row: &'ast ItemStruct) {
        self.found.extend(Recorded::marked(row).map(|recorded| recorded.map(|marked| marked.statements().to_vec())));
        syn::visit::visit_item_struct(self, row);
    }
}
