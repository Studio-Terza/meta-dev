mod token_paths;

use syn::visit::{self, Visit};
use syn::{ExprPath, ItemImpl, ItemMod, Macro, QSelf, Type};

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

pub(crate) struct Walker<'run> {
    file: &'run SourceFile,
    scopes: &'run Scopes,
    wanted: &'run Wanted,
    resolver: &'run Resolver<'run>,
    module: Module,
    ground: Ground,
    cover: Cover,
    found: Vec<Use>,
}

impl<'run> Walker<'run> {
    pub(crate) fn walk(
        file: &'run SourceFile,
        scopes: &'run Scopes,
        wanted: &'run Wanted,
        resolver: &'run Resolver<'run>,
    ) -> Vec<Use> {
        let ground = if Gate::PORTS.admits(file) { Ground::Ports } else { Ground::Open };
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

    fn made(&self, path: &UsePath, line: LineNumber) -> Option<Use> {
        self.wanted(path).then(|| {
            let candidates = self.resolver.candidates(&self.module, path);
            Use::new(Site::new(self.file.path(), line), candidates, self.ground, self.cover)
        })
    }

    fn wanted(&self, path: &UsePath) -> bool {
        match path.segments().split_last() {
            Some((last, [_, ..])) => self.wanted.holds(last),
            Some((_, [])) => self.scopes.renames_head(&self.module, path),
            None => false,
        }
    }

    fn written(expression: &ExprPath) -> UsePath {
        let path = UsePath::from(&expression.path);
        expression.qself.as_ref().map_or(path.clone(), |qself| Self::qualified(qself, &path))
    }

    fn qualified(qself: &QSelf, path: &UsePath) -> UsePath {
        match qself.ty.as_ref() {
            Type::Path(inner) if qself.position == 0 => {
                path.segments().iter().fold(UsePath::from(&inner.path), |full, segment| full.extended(segment.clone()))
            }
            _ => UsePath::default(),
        }
    }
}

impl<'ast> Visit<'ast> for Walker<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let outer = self.module.clone();
        let ground = self.ground;
        self.module = outer.within(&Name::from(&module.ident));
        self.ground = if Gate::PORTS.names(module) { Ground::Ports } else { ground };
        visit::visit_item_mod(self, module);
        self.module = outer;
        self.ground = ground;
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        let outer = self.cover;
        let guarded = match item.self_ty.as_ref() {
            Type::Path(path) => path
                .path
                .segments
                .last()
                .is_some_and(|last| self.scopes.guarded(&self.module, &Name::from(&last.ident))),
            _ => false,
        };
        self.cover = if guarded { Cover::Guarded } else { outer };
        visit::visit_item_impl(self, item);
        self.cover = outer;
    }

    fn visit_expr_path(&mut self, expression: &'ast ExprPath) {
        let path = Self::written(expression);
        let line = expression.path.segments.first().map(|segment| LineNumber::from(segment.ident.span()));
        let made: Vec<Use> = line
            .filter(|_| !path.segments().is_empty())
            .into_iter()
            .filter_map(|line| self.made(&path, line))
            .collect();
        self.found.extend(made);
        visit::visit_expr_path(self, expression);
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        let made: Vec<Use> = TokenPaths::within(invocation.tokens.clone())
            .iter()
            .filter_map(|spelled| self.made(spelled.path(), spelled.started_on()))
            .collect();
        self.found.extend(made);
    }
}
