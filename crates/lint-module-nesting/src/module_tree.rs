mod finding;
mod module_path;

use finding::ReachesPastModRs;
use module_path::{ModuleIndex, ModulePath, Target};

use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_layout::UseEdges;
use lint_layout::UsePath;

pub struct ModuleTree<'file> {
    modules: ModuleIndex<'file>,
    deep_uses: Vec<Violation<'file>>,
}

impl<'file> From<&[&'file SourceFile]> for ModuleTree<'file> {
    fn from(files: &[&'file SourceFile]) -> ModuleTree<'file> {
        let empty = ModuleTree {
            modules: ModuleIndex::from(files.iter().map(|file| ModulePath::of_file(file)).collect::<Vec<_>>()),
            deep_uses: Vec::new(),
        };
        files.iter().fold(empty, |tree, file| tree.absorbed(file))
    }
}

impl<'file> ModuleTree<'file> {
    pub fn violations(&self) -> impl Iterator<Item = Violation<'file>> + '_ {
        self.deep_uses.iter().cloned()
    }

    fn absorbed(mut self, file: &'file SourceFile) -> ModuleTree<'file> {
        let module = ModulePath::of_file(file);
        UseEdges::each(file.syntax(), |path| self.absorb(module, path));
        self
    }

    fn absorb(&mut self, module: ModulePath<'file>, path: &UsePath) {
        let target = Target::from_use(&module, path, &self.modules);
        let crossed = target.as_ref().and_then(|target| target.crossed_directory(&module, &self.modules));
        self.deep_uses
            .extend(crossed.map(|target| Violation::new(Site::of_file(module.file()), ReachesPastModRs::from(target))));
    }
}
