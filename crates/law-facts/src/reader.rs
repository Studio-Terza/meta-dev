mod mentions;
mod routes;
mod scope;

pub(crate) use scope::Import;

use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Block, ItemImpl, ItemMod, Macro};

use crate::kind::Kind;
use crate::law::{Law, Origin};
use crate::package::Package;
use crate::traits::Traits;
use lint_core::{Site, SourceFile};
use lint_layout::{LineNumber, Name};
use mentions::Mentions;
use routes::{Routes, Within};
use scope::Scope;

pub(crate) struct Reader<'file, 'traits> {
    file: &'file SourceFile,
    traits: &'traits Traits<'file>,
    scopes: Vec<Scope>,
    laws: Vec<Law<'file>>,
}

impl<'file, 'traits> Reader<'file, 'traits> {
    pub(crate) fn read(file: &'file SourceFile, traits: &'traits Traits<'file>) -> Vec<Law<'file>> {
        let outer = Scope::default().within(file.syntax().items.iter());
        let mut reader = Reader { file, traits, scopes: Vec::from([outer]), laws: Vec::new() };
        reader.visit_file(file.syntax());
        reader.laws
    }

    fn scope(&self) -> Scope {
        self.scopes.last().cloned().unwrap_or_default()
    }

    fn law(&self, invocation: &Macro, kind: Kind) -> Law<'file> {
        let scope = self.scope();
        let within = Within::new(&scope, self.traits, self.file.placement().package().crate_name());
        let name = invocation.path.segments.last().map(|last| Name::from(&last.ident)).unwrap_or_default();
        let site = Site::new(self.file.path(), LineNumber::from(invocation.path.span()));
        Law::new(
            Origin::new(Package::containing(self.file), site, name),
            kind,
            Mentions::beyond_title(invocation.tokens.clone(), kind),
            Routes::read(invocation, &within, kind),
        )
    }
}

impl<'ast> Visit<'ast> for Reader<'_, '_> {
    fn visit_item_mod(&mut self, item: &'ast ItemMod) {
        let inner = item.content.as_ref().map_or_else(|| self.scope(), |(_, items)| self.scope().within(items.iter()));
        self.scopes.push(inner);
        visit::visit_item_mod(self, item);
        self.scopes.pop();
    }

    fn visit_block(&mut self, block: &'ast Block) {
        self.scopes.push(self.scope().within_statements(&block.stmts));
        visit::visit_block(self, block);
        self.scopes.pop();
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        self.scopes.push(self.scope().implementing(&item.self_ty));
        visit::visit_item_impl(self, item);
        self.scopes.pop();
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        let kind = Kind::of_macro(invocation);
        let law = (kind != Kind::Other).then(|| self.law(invocation, kind));
        self.laws.extend(law);
        visit::visit_macro(self, invocation);
    }
}
