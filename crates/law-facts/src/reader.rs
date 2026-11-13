mod mentions;
mod routes;
mod scope;

use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Block, ItemImpl, ItemMod, Macro};

use crate::kind::Kind;
use crate::law::{Law, Origin};
use crate::package::Package;
use crate::traits::Traits;
use lint_core::{FileHead, Parsed, Site};
use lint_layout::{LineNumber, Name};
use mentions::Mentions;
use routes::{Routes, Within};
use scope::Scope;

pub(crate) struct Reader<'file, 'traits> {
    head: &'file FileHead,
    traits: &'traits Traits<'file>,
    scopes: Vec<Scope>,
    laws: Vec<Law<'file>>,
}

impl<'file, 'traits> Reader<'file, 'traits> {
    pub(crate) fn read(parsed: Parsed<'_, 'file>, traits: &'traits Traits<'file>) -> Vec<Law<'file>> {
        let syntax = parsed.file().syntax();
        let outer = Scope::default().within(syntax.items.iter());
        let mut reader = Reader { head: parsed.head(), traits, scopes: Vec::from([outer]), laws: Vec::new() };
        reader.visit_file(syntax);
        reader.laws
    }

    fn scope(&self) -> Scope {
        self.scopes.last().cloned().unwrap_or_default()
    }

    fn law(&self, invocation: &Macro, kind: Kind) -> Law<'file> {
        let scope = self.scope();
        let within = Within::new(&scope, self.traits, self.head.placement().package().crate_name());
        let name = invocation.path.segments.last().map_or_default(|last| Name::from(&last.ident));
        let site = Site::new(self.head.path(), LineNumber::from(invocation.path.span()));
        Law::new(
            Origin::new(Package::from(self.head), site, name),
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
