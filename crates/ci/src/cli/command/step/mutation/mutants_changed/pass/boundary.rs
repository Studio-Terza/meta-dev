use std::collections::{BTreeMap, BTreeSet};
use std::iter;
use std::path::{Path, PathBuf};

use file_ports::Files;
use text_lines::Lines;

use super::super::super::kept::Mutant;
use crate::cli::command::failure::Failure;
use crate::cli::ignored::Ignored;
use crate::cli::ports::Text;
use invariants::Ignoring;

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Narrowed(BTreeMap<PathBuf, BTreeSet<u32>>);

impl Narrowed {
    pub(super) fn diff(&self, files: &impl Files) -> Result<Text, Failure<'static>> {
        let paths: Vec<&PathBuf> = self.0.keys().collect();
        let patched = self
            .0
            .iter()
            .zip(files.read_text_many(&paths))
            .map(|((file, lines), source)| {
                let read = source.map_err(|failure| Failure::unusable(file.clone(), failure))?;
                let numbered: Vec<String> = read.as_ref().parse::<Lines>().into_iter().flatten().collect();
                Ok(Self::patched(file, lines, &numbered))
            })
            .collect::<Result<Vec<String>, Failure<'static>>>()?;
        Ok(Text::new(patched.concat()))
    }

    fn patched(file: &Path, lines: &BTreeSet<u32>, numbered: &[String]) -> String {
        let header = format!("--- a/{0}\n+++ b/{0}\n", file.display());
        let hunks = lines.iter().filter_map(|line| {
            let text = usize::try_from(*line)
                .ignoring(Ignored::BeyondRange)
                .and_then(|found| numbered.get(found.checked_sub(1)?))?;
            Some(format!("@@ -{line},1 +{line},1 @@\n-{text}\n+{text}\n"))
        });
        iter::once(header).chain(hunks).collect()
    }
}

impl<'mutant> FromIterator<&'mutant Mutant> for Narrowed {
    fn from_iter<Mutants: IntoIterator<Item = &'mutant Mutant>>(mutants: Mutants) -> Narrowed {
        Narrowed(mutants.into_iter().fold(BTreeMap::new(), |mut by: BTreeMap<PathBuf, BTreeSet<u32>>, mutant| {
            by.entry(mutant.file().to_path_buf()).or_default().insert(mutant.line());
            by
        }))
    }
}
