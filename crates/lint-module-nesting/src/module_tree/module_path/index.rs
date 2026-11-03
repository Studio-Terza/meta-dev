use std::collections::{HashMap, HashSet};

use lint_layout::Name;
use lint_placement::CrateRoot;

use super::ModulePath;

#[derive(Debug)]
pub struct ModuleIndex {
    modules: Vec<ModulePath>,
    latest: HashMap<CrateRoot, HashMap<Vec<Name>, ModulePath>>,
    directories: HashMap<CrateRoot, HashSet<Vec<Name>>>,
}

impl From<Vec<ModulePath>> for ModuleIndex {
    fn from(modules: Vec<ModulePath>) -> ModuleIndex {
        let latest = modules.iter().fold(
            HashMap::new(),
            |mut found: HashMap<CrateRoot, HashMap<Vec<Name>, ModulePath>>, module| {
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

impl ModuleIndex {
    pub fn modules(&self) -> &[ModulePath] {
        &self.modules
    }

    pub fn same_as(&self, probe: &ModulePath) -> Option<&ModulePath> {
        self.latest.get(&probe.crate_root).and_then(|found| found.get(probe.segments.as_slice()))
    }

    pub fn directory_same_as<'probe>(&self, probe: &'probe ModulePath) -> Option<&'probe ModulePath> {
        self.directories.get(&probe.crate_root).filter(|found| found.contains(probe.segments.as_slice())).map(|_| probe)
    }
}
