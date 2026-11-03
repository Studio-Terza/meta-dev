use std::collections::{HashMap, HashSet};

use lint_layout::Name;
use lint_placement::CrateRoot;

use super::ModulePath;

#[derive(Debug)]
pub struct ModuleIndex<'file> {
    modules: Vec<ModulePath<'file>>,
    latest: HashMap<CrateRoot, HashMap<Vec<Name>, ModulePath<'file>>>,
    directories: HashMap<CrateRoot, HashSet<Vec<Name>>>,
}

impl<'file> From<Vec<ModulePath<'file>>> for ModuleIndex<'file> {
    fn from(modules: Vec<ModulePath<'file>>) -> ModuleIndex<'file> {
        let latest = modules.iter().fold(
            HashMap::new(),
            |mut found: HashMap<CrateRoot, HashMap<Vec<Name>, ModulePath<'file>>>, module| {
                found.entry(module.crate_root.clone()).or_default().insert(module.segments.clone(), module.clone());
                found
            },
        );
        let directories = modules.iter().filter_map(ModulePath::directory_module).fold(
            HashMap::new(),
            |mut found: HashMap<CrateRoot, HashSet<Vec<Name>>>, module| {
                found.entry(module.crate_root.clone()).or_default().insert(module.segments.clone());
                found
            },
        );
        ModuleIndex { modules, latest, directories }
    }
}

impl<'file> ModuleIndex<'file> {
    pub fn modules(&self) -> &[ModulePath<'file>] {
        &self.modules
    }

    pub fn same_as(&self, probe: &ModulePath<'file>) -> Option<&ModulePath<'file>> {
        self.latest.get(&probe.crate_root).and_then(|found| found.get(probe.segments.as_slice()))
    }

    pub fn directory_same_as<'probe>(&self, probe: &'probe ModulePath<'file>) -> Option<&'probe ModulePath<'file>> {
        self.directories.get(&probe.crate_root).filter(|found| found.contains(probe.segments.as_slice())).map(|_| probe)
    }
}
