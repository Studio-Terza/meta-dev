use syn::visit::{self, Visit};
use syn::{Block, ItemUse, Path, Stmt, UseTree, token};

#[derive(Debug, Default)]
pub struct Mentioned {
    paths: Vec<Path>,
    uses: Vec<UseTree>,
}

impl Mentioned {
    #[must_use]
    pub fn in_statements(statements: &[Stmt]) -> Mentioned {
        let mut found = Mentioned::default();
        found.visit_block(&Block { brace_token: token::Brace::default(), stmts: statements.to_vec() });
        found
    }

    #[must_use]
    pub fn paths(&self) -> &[Path] {
        &self.paths
    }

    #[must_use]
    pub fn uses(&self) -> &[UseTree] {
        &self.uses
    }
}

impl<'ast> Visit<'ast> for Mentioned {
    fn visit_path(&mut self, path: &'ast Path) {
        self.paths.push(path.clone());
        visit::visit_path(self, path);
    }

    fn visit_item_use(&mut self, item: &'ast ItemUse) {
        self.uses.push(item.tree.clone());
    }
}
