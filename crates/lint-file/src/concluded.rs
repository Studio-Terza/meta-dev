use std::collections::BTreeMap;
use std::path::Path;

use lint_core::{SourceFile, Violation};

#[derive(Debug, Default)]
pub struct Concluded<'head>(BTreeMap<&'head Path, Vec<Violation<'head>>>);

impl<'head> Concluded<'head> {
    #[must_use]
    pub fn found_in<'file>(&self, file: &SourceFile) -> Vec<Violation<'file>>
    where
        'head: 'file,
    {
        let paths = std::iter::once(file).chain(file.test_modules()).map(|found| found.head().path());
        paths.filter_map(|path| self.0.get(path)).flatten().cloned().collect()
    }
}

impl<'head> FromIterator<Violation<'head>> for Concluded<'head> {
    fn from_iter<Found: IntoIterator<Item = Violation<'head>>>(found: Found) -> Concluded<'head> {
        Concluded(found.into_iter().fold(
            BTreeMap::new(),
            |mut filed: BTreeMap<&'head Path, Vec<Violation<'head>>>, violation| {
                filed.entry(violation.site().path()).or_default().push(violation);
                filed
            },
        ))
    }
}

#[derive(Debug, Default)]
pub struct Listed<'head> {
    at: Option<&'head Path>,
    violations: Vec<Violation<'head>>,
}

impl<'head> Listed<'head> {
    #[must_use]
    pub fn new(files: impl IntoIterator<Item = &'head Path>, violations: Vec<Violation<'head>>) -> Listed<'head> {
        Listed { at: files.into_iter().min(), violations }
    }

    #[must_use]
    pub fn found_in<'file>(&self, file: &SourceFile) -> Vec<Violation<'file>>
    where
        'head: 'file,
    {
        self.at.filter(|at| *at == file.head().path()).map_or_default(|_| self.violations.clone())
    }
}
