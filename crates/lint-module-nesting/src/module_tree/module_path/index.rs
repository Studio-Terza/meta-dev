use std::collections::{HashMap, HashSet};

use lint_layout::Name;
use lint_placement::CrateRoot;

use super::{ModulePath, Same};

#[derive(Debug)]
pub struct ModuleIndex {
    modules: Vec<ModulePath>,
    last_index: HashMap<CrateRoot, HashMap<Vec<Name>, usize>>,
    directories: HashMap<CrateRoot, HashSet<Vec<Name>>>,
}

impl From<Vec<ModulePath>> for ModuleIndex {
    fn from(modules: Vec<ModulePath>) -> ModuleIndex {
        let last_index = modules.iter().enumerate().fold(
            HashMap::new(),
            |mut found: HashMap<CrateRoot, HashMap<Vec<Name>, usize>>, (index, module)| {
                found.entry(module.crate_root.clone()).or_default().insert(module.segments.clone(), index);
                found
            },
        );
        let directories = modules.iter().filter(|module| module.is_directory_module()).fold(
            HashMap::new(),
            |mut found: HashMap<CrateRoot, HashSet<Vec<Name>>>, module| {
                found.entry(module.crate_root.clone()).or_default().insert(module.segments.clone());
                found
            },
        );
        ModuleIndex { modules, last_index, directories }
    }
}

impl ModuleIndex {
    pub fn modules(&self) -> &[ModulePath] {
        &self.modules
    }

    pub fn same_as(&self, probe: &ModulePath) -> Option<&ModulePath> {
        self.last_index
            .get(&probe.crate_root)
            .and_then(|positions| positions.get(probe.segments.as_slice()))
            .and_then(|index| self.modules.get(*index))
    }

    pub fn directory_same_as(&self, probe: &ModulePath) -> bool {
        self.directories.get(&probe.crate_root).is_some_and(|found| found.contains(probe.segments.as_slice()))
    }

    pub fn deepest_containing(&self, target: &ModulePath, user: &ModulePath) -> Option<&ModulePath> {
        (0..=target.segments.len())
            .rev()
            .map(|length| target.truncated_to_length(length))
            .filter(|probe| !probe.relates::<Same>(user))
            .find_map(|probe| self.same_as(&probe))
    }
}
