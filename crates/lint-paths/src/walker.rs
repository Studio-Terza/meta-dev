mod token_paths;

use syn::visit::{self, Visit};
use syn::{ExprPath, ItemImpl, ItemMod, Macro, Path, QSelf, Type};

use crate::cover::Cover;
use crate::ground::Ground;
use crate::module::Module;
use crate::resolver::Resolver;
use crate::scopes::Scopes;
use crate::uses::Use;
use crate::wanted::Wanted;
use lint_core::{Gate, Site, SourceFile};
use lint_layout::{LineNumber, Name, UsePath};
use token_paths::TokenPaths;

pub(crate) struct Walker<'file, 'run> {
    file: &'file SourceFile,
    scopes: &'run Scopes<'run>,
    wanted: &'run Wanted<'run>,
    resolver: &'run Resolver<'run>,
    module: Module<'file>,
    ground: Ground,
    cover: Cover,
    found: Vec<Use<'file>>,
}

impl<'file, 'run> Walker<'file, 'run> {
    pub(crate) fn walk(
        file: &'file SourceFile,
        scopes: &'run Scopes<'run>,
        wanted: &'run Wanted<'run>,
        resolver: &'run Resolver<'run>,
    ) -> Vec<Use<'file>> {
        let ground = Gate::PORTS.admits(file).map_or(Ground::Open, |_| Ground::Ports);
        let mut walker = Walker {
            file,
            scopes,
            wanted,
            resolver,
            module: Module::of_file(file),
            ground,
            cover: Cover::Bare,
            found: Vec::new(),
        };
        walker.visit_file(file.syntax());
        walker.found
    }

    fn made(&self, path: UsePath, line: LineNumber) -> Option<Use<'file>> {
        self.wanted(&path).map(|_| ()).map(|()| {
            let candidates = self.resolver.candidates(&self.module, path);
            Use::new(Site::new(self.file.path(), line), candidates, self.ground, self.cover)
        })
    }

    fn wanted<'path>(&self, path: &'path UsePath) -> Option<&'path UsePath> {
        match path.segments().split_last() {
            Some((last, [_, ..])) => self.wanted.holds(last).map(|_| path),
            Some((_, [])) => self.scopes.renames_head(&self.module, path).map(|_| path),
            None => None,
        }
    }

    fn written(expression: &ExprPath) -> UsePath {
        let path = UsePath::from(&expression.path);
        expression.qself.as_ref().map(|qself| Self::qualified(qself, &path)).unwrap_or(path)
    }

    fn qualified(qself: &QSelf, path: &UsePath) -> UsePath {
        Self::type_path(&qself.ty)
            .filter(|_| qself.position == 0)
            .map_or_else(UsePath::default, |inner| UsePath::from(inner).joined(path.segments()))
    }

    pub(crate) fn type_path(ty: &Type) -> Option<&Path> {
        match ty {
            Type::Path(inner) => Some(&inner.path),
            _ => None,
        }
    }
}

impl<'ast, 'file> Visit<'ast> for Walker<'file, '_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let ground = self.ground;
        self.module.enter(Name::from(&module.ident));
        self.ground = Gate::PORTS.names(module).map_or(ground, |_| Ground::Ports);
        visit::visit_item_mod(self, module);
        self.module.leave();
        self.ground = ground;
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        let outer = self.cover;
        let guarded = Self::type_path(&item.self_ty)
            .and_then(|path| path.segments.last())
            .and_then(|last| self.scopes.guarded(&self.module, &Name::from(&last.ident)));
        self.cover = guarded.map_or(outer, |_| Cover::Guarded);
        visit::visit_item_impl(self, item);
        self.cover = outer;
    }

    fn visit_expr_path(&mut self, expression: &'ast ExprPath) {
        let path = Self::written(expression);
        let line = expression.path.segments.first().map(|segment| LineNumber::from(segment.ident.span()));
        let made = path.segments().first().and(line).and_then(|line| self.made(path, line));
        self.found.extend(made);
        visit::visit_expr_path(self, expression);
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        let made: Vec<Use<'file>> = TokenPaths::within(invocation.tokens.clone())
            .into_iter()
            .filter_map(|spelled| spelled.handed(|path, line| self.made(path, line)))
            .collect();
        self.found.extend(made);
    }
}
