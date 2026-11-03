use std::collections::{HashMap, HashSet};
use std::iter;

use lint_layout::Name;
use lint_placement::CrateRoot;

use super::ModulePath;
use super::ancestor::Ancestor;
use super::relation::Relation;

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
        let directories = modules
            .iter()
            .filter(|module| module.file.file_name().is_some_and(|name| name == Self::MODULE_FILE))
            .fold(HashMap::new(), |mut found: HashMap<CrateRoot, HashSet<Vec<Name>>>, module| {
                found.entry(module.crate_root.clone()).or_default().insert(module.segments.clone());
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

    pub fn deepest(&self, target: &ModulePath<'file>) -> Option<&ModulePath<'file>> {
        let found = self.latest.get(&target.crate_root)?;
        iter::successors(Some(target.clone()), ModulePath::parent)
            .find_map(|probe| found.get(probe.segments.as_slice()))
    }

    pub fn crossed(&self, target: &ModulePath<'file>, user: &ModulePath<'file>) -> Option<ModulePath<'file>> {
        let directories = self.directories.get(&target.crate_root)?;
        let mut boundaries: Vec<ModulePath<'file>> = iter::successors(target.parent(), ModulePath::parent)
            .filter(|boundary| !boundary.segments.is_empty())
            .collect();
        boundaries.reverse();
        boundaries.into_iter().find(|boundary| {
            directories.contains(boundary.segments.as_slice()) && Ancestor::holds(boundary, user).is_none()
        })
    }
}
