mod context;
mod finding;
mod per_element;
mod scope;

use std::collections::BTreeMap;
use std::mem;

use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    Attribute, Block, Expr, ExprClosure, ExprField, ExprForLoop, ExprLoop, ExprMethodCall, ExprPath, ExprWhile,
    ImplItemFn, ItemFn, ItemImpl, ItemMod, ItemTrait, Macro, Signature, TraitItemFn,
};

use lint_core::{Macros, SourceFile, Violation, Walk};
use lint_core::{Module, Nested};
use lint_layout::{LineNumber, Name};
use lint_placement::CrateName;

use crate::holders::Holders;
use crate::judge::Judge;
use context::Context;
use finding::{Looping, Port, Spot, Within};
use per_element::PerElement;
use scope::Scope;

pub struct Reader<'run, 'file> {
    file: &'file SourceFile,
    module: Module<'file>,
    context: Context<'run>,
    scope: Scope<'run>,
    found: BTreeMap<Spot, Within>,
}

impl<'run, 'file> Reader<'run, 'file> {
    pub fn violations(
        file: &'file SourceFile,
        judge: &'run Judge<'run>,
        holders: &'run Holders,
    ) -> Vec<Violation<'file>> {
        let crate_name = CrateName::from(file.head().placement().crate_root());
        let mut reader = Reader {
            file,
            module: Module::of_file(file),
            context: Context::new(judge, holders, crate_name),
            scope: Scope::default(),
            found: BTreeMap::new(),
        };
        reader.visit_file(file.syntax());
        reader.found.into_iter().map(|(spot, within)| spot.violation(file.head().path(), within)).collect()
    }

    fn scoped(&mut self, scope: Scope<'run>, visit: impl FnOnce(&mut Self)) {
        let outer = mem::replace(&mut self.scope, scope);
        visit(self);
        self.scope = outer;
    }

    fn within(&mut self, within: impl FnOnce(Option<Within>) -> Option<Within>, visit: impl FnOnce(&mut Self)) {
        let inner = within(self.scope.within());
        let outer = self.scope.replace_within(inner);
        visit(self);
        self.scope.replace_within(outer);
    }

    fn function(&mut self, attributes: &[Attribute], signature: &Signature, body: &Block) {
        Walk::enter(attributes, || {
            let declared = self.context.judge().generics(&self.module, &signature.generics);
            let generics = self.scope.generics().iter().cloned().chain(declared).collect();
            let bindings = self.context.bindings(&self.module, &generics, signature);
            let scope = self.scope.function(generics, bindings);
            self.scoped(scope, |reader| reader.visit_block(body));
        });
    }

    fn found(&mut self, port: Option<Port>, line: LineNumber) {
        let within = self.scope.within();
        self.found.extend(within.zip(port).map(|(scope, found)| (Spot::new(line, found), scope)));
    }
}

impl<'file> Nested<'file> for Reader<'_, 'file> {
    fn module(&mut self) -> &mut Module<'file> {
        &mut self.module
    }
}

impl<'ast> Visit<'ast> for Reader<'_, '_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        self.nested(module, |reader| visit::visit_item_mod(reader, module));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        Walk::enter(&item.attrs, || {
            let generics = self.context.judge().generics(&self.module, &item.generics);
            let scope = Scope::declared(generics, self.context.held(&item.self_ty));
            self.scoped(scope, |reader| visit::visit_item_impl(reader, item));
        });
    }

    fn visit_item_trait(&mut self, item: &'ast ItemTrait) {
        Walk::enter(&item.attrs, || {
            let generics = self.context.judge().generics(&self.module, &item.generics);
            let scope = Scope::declared(generics, None);
            self.scoped(scope, |reader| visit::visit_item_trait(reader, item));
        });
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        self.function(&function.attrs, &function.sig, &function.block);
    }

    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        self.function(&function.attrs, &function.sig, &function.block);
    }

    fn visit_trait_item_fn(&mut self, function: &'ast TraitItemFn) {
        function.default.iter().for_each(|body| self.function(&function.attrs, &function.sig, body));
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Macros::walk(self.file, invocation, |statement| self.visit_stmt(statement));
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        let receiver = self.scope.reached_receiver(&call.receiver);
        let paced = receiver.and_then(|_| self.context.judge().ports().paces(&Name::from(&call.method))).map(|_| ());
        self.within(|outer| outer.filter(|_| paced.is_none()), |reader| reader.visit_expr(&call.receiver));
        let within = PerElement::adaptor(call).map(Within::Closure);
        call.args.iter().for_each(|argument| {
            let closure = within.filter(|_| matches!(argument, Expr::Closure(_)));
            self.within(|outer| outer.or(closure), |reader| reader.visit_expr(argument));
        });
    }

    fn visit_expr_closure(&mut self, closure: &'ast ExprClosure) {
        let outer = self.scope.shadowed(closure);
        self.visit_expr(&closure.body);
        self.scope.unshadowed(outer);
    }

    fn visit_expr_for_loop(&mut self, expr: &'ast ExprForLoop) {
        self.visit_expr(&expr.expr);
        self.within(|outer| outer.or(Some(Within::Loop(Looping::For))), |reader| reader.visit_block(&expr.body));
    }

    fn visit_expr_while(&mut self, expr: &'ast ExprWhile) {
        self.visit_expr(&expr.cond);
        self.within(|outer| outer.or(Some(Within::Loop(Looping::While))), |reader| reader.visit_block(&expr.body));
    }

    fn visit_expr_loop(&mut self, expr: &'ast ExprLoop) {
        self.within(|outer| outer.or(Some(Within::Loop(Looping::Bare))), |reader| reader.visit_block(&expr.body));
    }

    fn visit_expr_path(&mut self, path: &'ast ExprPath) {
        self.found(self.scope.reached_path(path), LineNumber::from(path.path.span()));
    }

    fn visit_expr_field(&mut self, field: &'ast ExprField) {
        match self.scope.reached_field(field) {
            Some(port) => self.found(Some(port), LineNumber::from(field.member.span())),
            None => visit::visit_expr_field(self, field),
        }
    }
}
