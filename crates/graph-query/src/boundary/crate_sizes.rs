use std::collections::BTreeMap;
use std::fmt;

use lint_core::SourceFile;
use workspace_metadata::{Chained, Graph, Metadata};

#[derive(Clone, Copy, Debug, Default)]
struct Size {
    files: usize,
    lines: usize,
}

#[derive(Debug)]
pub struct CrateSizes<'files> {
    sizes: BTreeMap<String, Size>,
    chained: Vec<Chained<'files>>,
}

impl<'files> CrateSizes<'files> {
    #[must_use]
    pub fn measured(files: &[SourceFile], workspaces: &'files [Metadata]) -> CrateSizes<'files> {
        let grouped = files.iter().fold(BTreeMap::new(), |mut grouped: BTreeMap<String, Vec<&SourceFile>>, file| {
            let root = file.head().placement().package().root();
            let named = root.package().map(ToString::to_string);
            grouped
                .entry(
                    named.unwrap_or_else(|| root.directory_name().unwrap_or_default().to_string_lossy().into_owned()),
                )
                .or_default()
                .push(file);
            grouped
        });
        let sizes = grouped
            .into_iter()
            .map(|(name, found)| {
                let lines = found.iter().map(|file| file.text().as_ref().matches('\n').count()).sum();
                (name, Size { files: found.len(), lines })
            })
            .collect();
        CrateSizes { sizes, chained: workspaces.iter().collect::<Graph<'files>>().chained() }
    }
}

impl fmt::Display for CrateSizes<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut rows: Vec<(Size, &Chained<'_>)> = self
            .chained
            .iter()
            .map(|chained| (self.sizes.get(&chained.name().to_string()).copied().unwrap_or_default(), chained))
            .collect();
        rows.sort_by_key(|(size, chained)| (std::cmp::Reverse(size.lines), chained.name().to_string()));
        writeln!(formatter, "crates by size and place on the compile chain:")?;
        rows.iter().try_for_each(|(size, chained)| {
            writeln!(formatter, "  {}: {} files, {} lines; {chained}", chained.name(), size.files, size.lines)
        })
    }
}
