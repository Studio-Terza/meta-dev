mod finding;
mod module_path;
mod reexport_count;

use std::collections::BTreeMap;

use super::ModuleNesting;
use finding::Finding;
use module_path::{ModuleIndex, ModulePath, Target};

use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_layout::UseEdges;
use lint_layout::UsePath;
use reexport_count::ReexportCount;

pub struct ModuleTree<'file> {
    modules: ModuleIndex<'file>,
    deep_uses: Vec<Violation<'file>>,
    reexports: BTreeMap<ModulePath<'file>, ReexportCount>,
}

impl<'file> From<&[&'file SourceFile]> for ModuleTree<'file> {
    fn from(files: &[&'file SourceFile]) -> ModuleTree<'file> {
        let empty = ModuleTree {
            modules: ModuleIndex::from(files.iter().map(|file| ModulePath::of_file(file)).collect::<Vec<_>>()),
            deep_uses: Vec::new(),
            reexports: BTreeMap::new(),
        };
        files.iter().fold(empty, |tree, file| tree.absorbed(file))
    }
}

impl<'file> ModuleTree<'file> {
    pub fn violations(&self) -> impl Iterator<Item = Violation<'file>> + '_ {
        let most = ModuleNesting::REEXPORTS;
        let deep_uses = self.deep_uses.iter().cloned();
        let crowded = self
            .reexports
            .iter()
            .filter(|(module, _)| module.parent().is_some())
            .filter_map(move |(module, count)| count.beyond(most).map(|count| (module, count)))
            .map(move |(module, count)| {
                Violation::new(Site::of_file(module.file()), Finding::CrowdedReexports { count, most })
            });
        deep_uses.chain(crowded)
    }

    fn absorbed(mut self, file: &'file SourceFile) -> ModuleTree<'file> {
        let module = ModulePath::of_file(file);
        self.reexports.insert(module, ReexportCount::of_file(file.syntax()));
        UseEdges::each(file.syntax(), |path| self.absorb(module, path));
        self
    }

    fn absorb(&mut self, module: ModulePath<'file>, path: &UsePath) {
        let target = Target::from_use(&module, path, &self.modules);
        let crossed = target.as_ref().and_then(|target| target.crossed_directory(&module, &self.modules));
        self.deep_uses.extend(
            crossed.map(|target| Violation::new(Site::of_file(module.file()), Finding::ReachesPastModRs { target })),
        );
    }
}
