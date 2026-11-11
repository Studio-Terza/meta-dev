mod declarations;

use std::collections::BTreeSet;

use lint_core::{Site, SourceFile, Violation};
use lint_effect_sites::CrateEffects;
use lint_layout::Name;

use declarations::Declarations;

#[derive(Debug)]
pub(crate) struct PortCrate<'file> {
    files: Vec<&'file SourceFile>,
    fillers: BTreeSet<Site<'file>>,
}

impl<'file> PortCrate<'file> {
    pub(crate) fn declared(files: &[&'file SourceFile]) -> Option<PortCrate<'file>> {
        let ports: BTreeSet<Name> = files.iter().flat_map(|file| Declarations::ports(file)).collect();
        ports.first().map(|_| PortCrate {
            files: files.to_vec(),
            fillers: files.iter().flat_map(|file| Declarations::fillers(file, &ports)).collect(),
        })
    }

    pub(crate) fn stray_effects(&self) -> Vec<Violation<'file>> {
        CrateEffects::found(&self.files).outside_fillers(&self.fillers)
    }
}
