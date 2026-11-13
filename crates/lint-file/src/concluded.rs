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
        self.0.get(file.head().path()).cloned().unwrap_or_default()
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
