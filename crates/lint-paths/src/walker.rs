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

    fn made(&self, path: &UsePath, line: LineNumber) -> Option<Use> {
        self.wanted(path).map(|_| {
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
        self.ground = Gate::PORTS.names(module).map_or(ground, |_| Ground::Ports);
        visit::visit_item_mod(self, module);
        self.module = outer;
        self.ground = ground;
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        let outer = self.cover;
        let guarded = match item.self_ty.as_ref() {
            Type::Path(path) => {
                path.path.segments.last().and_then(|last| self.scopes.guarded(&self.module, &Name::from(&last.ident)))
            }
            _ => None,
        };
        self.cover = guarded.map_or(outer, |_| Cover::Guarded);
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
