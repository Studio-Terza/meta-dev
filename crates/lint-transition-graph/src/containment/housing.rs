use std::collections::BTreeMap;
use std::path::Path;

use lint_core::{BinaryFiles, SourceFile};

use super::cluster::Cluster;
use super::unit::Unit;

#[derive(Debug)]
pub(crate) struct Housing<'found> {
    files: Vec<&'found SourceFile>,
    units: BTreeMap<&'found Path, Unit<'found>>,
}

impl<'found> Housing<'found> {
    pub(crate) fn by_target(files: &'found [SourceFile]) -> Housing<'found> {
        let shipped: Vec<&'found SourceFile> = files.iter().filter_map(SourceFile::shipped).collect();
        let binaries = BinaryFiles::collect(files);
        let paths: BTreeMap<&Path, &'found Path> = shipped.iter().map(|file| (file.path(), file.path())).collect();
        let units = shipped
            .iter()
            .map(|file| {
                let krate = file.placement().package().crate_name();
                let root = binaries.root_of(file.path()).and_then(|root| paths.get(root).copied());
                (file.path(), root.map_or_else(|| Unit::library(krate), |root| Unit::binary(krate, root)))
            })
            .collect();
        Housing { files: shipped, units }
    }

    pub(crate) fn files(&self) -> &[&'found SourceFile] {
        &self.files
    }

    pub(crate) fn unit(&self, file: &Path) -> Option<Unit<'found>> {
        self.units.get(file).copied()
    }

    pub(crate) fn homes(&self) -> BTreeMap<&'found Path, Cluster<'found>> {
        self.files
            .iter()
            .filter_map(|file| {
                self.unit(file.path())
                    .map(|unit| (file.path(), Cluster::module(unit, file.placement().module().segments())))
            })
            .collect()
    }
}
