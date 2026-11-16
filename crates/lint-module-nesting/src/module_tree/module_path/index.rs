use std::collections::{HashMap, HashSet};

use lint_layout::Name;
use lint_placement::CrateRoot;

use super::ModulePath;

#[derive(Debug)]
pub struct ModuleIndex<'file> {
    modules: Vec<ModulePath<'file>>,
    latest: HashMap<&'file CrateRoot, HashMap<Vec<&'file Name>, ModulePath<'file>>>,
    directories: HashMap<&'file CrateRoot, HashSet<Vec<&'file Name>>>,
}

impl<'file> From<Vec<ModulePath<'file>>> for ModuleIndex<'file> {
    fn from(modules: Vec<ModulePath<'file>>) -> ModuleIndex<'file> {
        let latest = modules.iter().filter_map(|module| module.nest.krate().map(|root| (root, module))).fold(
            HashMap::new(),
            |mut found: HashMap<&'file CrateRoot, HashMap<Vec<&'file Name>, ModulePath<'file>>>, (root, module)| {
                found.entry(root).or_default().insert(module.segments().iter().collect(), *module);
                found
            },
        );
        let directories = modules
            .iter()
            .filter(|module| module.file.file_name().is_some_and(|name| name == Self::MODULE_FILE))
            .filter_map(|module| module.nest.krate().map(|root| (root, module)))
            .fold(HashMap::new(), |mut found: HashMap<&'file CrateRoot, HashSet<Vec<&'file Name>>>, (root, module)| {
                found.entry(root).or_default().insert(module.segments().iter().collect());
                found
            });
        ModuleIndex { modules, latest, directories }
    }
}

impl<'file> ModuleIndex<'file> {
    const MODULE_FILE: &'static str = "mod.rs";

    pub fn modules(&self) -> &[ModulePath<'file>] {
        &self.modules
    }

    pub fn same_as(&self, crate_root: &CrateRoot, segments: &[&Name]) -> Option<ModulePath<'file>> {
        let found: &HashMap<Vec<&Name>, ModulePath<'file>> = self.latest.get(crate_root)?;
        found.get(segments).copied()
    }

    pub fn directory_same_as<'probe>(
        &self,
        crate_root: &CrateRoot,
        segments: &'probe [&'probe Name],
    ) -> Option<&'probe [&'probe Name]> {
        let found: &HashSet<Vec<&Name>> = self.directories.get(crate_root)?;
        found.contains(segments).then_some(segments)
    }
}
