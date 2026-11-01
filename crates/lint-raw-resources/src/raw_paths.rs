use std::collections::BTreeMap;

use syn::visit::{self, Visit};
use syn::{ExprPath, Ident, ItemMod, ItemUse, Macro, UseTree};

use lint_core::{Gate, Macros, Site, SourceFile, Violation};
use lint_layout::{LineNumber, Name, UsePath};

use super::Finding;

pub struct RawPaths<'file> {
    file: &'file SourceFile,
    imports: BTreeMap<Name, UsePath>,
    globs: Vec<UsePath>,
    found: Vec<Violation>,
}

impl RawPaths<'_> {
    const RAW: [(&str, &str); 3] = [("fs", "remove_dir_all"), ("mem", "forget"), ("env", "temp_dir")];
    const SELF: &str = "self";

    pub fn found(file: &SourceFile) -> Vec<Violation> {
        let mut paths = RawPaths { file, imports: BTreeMap::new(), globs: Vec::new(), found: Vec::new() };
        paths.visit_file(file.syntax());
        paths.found
    }

    fn imported(&mut self, prefix: &UsePath, tree: &UseTree) {
        match tree {
            UseTree::Path(path) => self.imported(&prefix.extended(Name::from(&path.ident)), &path.tree),
            UseTree::Name(name) => self.bound(prefix, &name.ident, None),
            UseTree::Rename(name) => self.bound(prefix, &name.ident, Some(&name.rename)),
            UseTree::Glob(_) => self.globs.push(prefix.clone()),
            UseTree::Group(group) => group.items.iter().for_each(|item| self.imported(prefix, item)),
        }
    }

    fn bound(&mut self, prefix: &UsePath, ident: &Ident, rename: Option<&Ident>) {
        let target = if ident == Self::SELF { prefix.clone() } else { prefix.extended(Name::from(ident)) };
        let local = rename.map(Name::from).or_else(|| target.segments().last().cloned());
        self.imports.extend(local.map(|local| (local, target)));
    }

    fn expansions(&self, path: &UsePath) -> Vec<UsePath> {
        let first = path.first().cloned().unwrap_or_default();
        let imported = self.imports.get(&first).map(|root| path.rerooted(root));
        let globbed = self.globs.iter().map(|glob| path.rerooted(&glob.extended(first.clone())));
        globbed.chain(imported).chain(Some(path.clone())).collect()
    }

    fn is_raw(path: &UsePath) -> bool {
        matches!(path.segments(), [.., module, function]
            if Self::RAW.iter().any(|(raw_module, raw_function)| module == raw_module && function == raw_function))
    }
}

impl<'ast> Visit<'ast> for RawPaths<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Some(module).filter(|module| !Gate::PORTS.names(module)).into_iter().for_each(|module| {
            visit::visit_item_mod(self, module);
        });
    }

    fn visit_item_use(&mut self, item: &'ast ItemUse) {
        self.imported(&UsePath::default(), &item.tree);
    }

    fn visit_expr_path(&mut self, expression: &'ast ExprPath) {
        let raw = self.expansions(&UsePath::from(&expression.path)).into_iter().find(Self::is_raw);
        let line = expression.path.segments.first().map(|segment| LineNumber::from(segment.ident.span()));
        let site = line.map(|line| Site::new(self.file.path(), line));
        self.found.extend(site.zip(raw).map(|(site, path)| Violation::new(site, Finding::from(path))));
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Macros::walk(invocation, |statement| self.visit_stmt(statement));
    }
}
