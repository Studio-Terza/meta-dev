use std::collections::BTreeMap;
use std::fmt;

use lint_core::SourceFile;
use workspace_metadata::{Chained, Graph, Metadata};

#[derive(Clone, Copy, Debug, Default)]
struct Size {
    files: usize,
    lines: usize,
}

impl std::ops::Add for Size {
    type Output = Size;

    fn add(self, other: Size) -> Size {
        Size { files: self.files + other.files, lines: self.lines + other.lines }
    }
}

#[derive(Debug)]
pub struct CrateSizes<'files> {
    sizes: BTreeMap<String, Size>,
    chained: Vec<Chained<'files>>,
}

impl<'files> CrateSizes<'files> {
    #[must_use]
    pub fn measured(files: &[SourceFile], workspaces: &'files [Metadata]) -> CrateSizes<'files> {
        let sizes = files.iter().fold(BTreeMap::new(), |mut sizes: BTreeMap<String, Size>, file| {
            let name = file.head().placement().package().root().package().map_or_default(ToString::to_string);
            let measured = Size { files: 1, lines: file.text().as_ref().matches('\n').count() };
            let held = sizes.remove(&name).unwrap_or_default();
            sizes.insert(name, held + measured);
            sizes
        });
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
