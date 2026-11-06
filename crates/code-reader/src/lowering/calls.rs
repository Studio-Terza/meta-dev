use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Expr, ExprCall, ExprMethodCall, ExprPath, File, Item, Macro, Token};

use code_model::{Building, Call, Callee, ItemPath, MacroCall, PathCall, Position, Region, Seat, TypeRef};
use lint_core::Syntax;
use lint_layout::{LineNumber, Name};

use crate::namespace::{Expansion, Resolving, ScopeId, StdNames, UsePath};

use super::Lowering;
use super::seat::{Built, Located};
use super::signature::Written;

#[derive(Debug)]
pub(super) enum CalleeRow {
    Method(Name),
    Path(ItemPath, TypeRef<Written>),
    Macro(MacroCall),
}

impl CalleeRow {
    fn built(self, resolving: &Resolving<'_, '_, '_>, scope: ScopeId) -> Callee {
        match self {
            CalleeRow::Method(name) => Callee::Method(name),
            CalleeRow::Path(written, owner) => {
                Callee::Path(PathCall::new(written, Written::typed(owner, resolving, scope)))
            }
            CalleeRow::Macro(invoked) => Callee::Macro(invoked),
        }
    }
}

#[derive(Debug)]
pub(super) struct CallRow {
    located: Located,
    callee: CalleeRow,
    region: Region,
    subject: Option<TypeRef<Written>>,
}

impl Built for CallRow {
    fn built(self, resolving: &Resolving<'_, '_, '_>, building: Building) -> Building {
        let scope = self.located.scope();
        let callee = self.callee.built(resolving, scope);
        let subject = self.subject.map(|subject| Written::typed(subject, resolving, scope));
        self.located.call(building, Call::new(callee, Seat::new(self.region, subject)))
    }
}

#[derive(Clone, Copy, Debug)]
struct Invocation<'invoked> {
    invoked: &'invoked Macro,
}

impl Invocation<'_> {
    fn arguments(self) -> Vec<Expr> {
        self.invoked.parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated).into_iter().flatten().collect()
    }

    fn items(self) -> Vec<Item> {
        self.invoked.parse_body::<File>().map(|file| file.items).unwrap_or_default()
    }
}

impl Lowering<'_, '_> {
    pub(super) fn method_called(&mut self, call: &ExprMethodCall) {
        self.called(LineNumber::from(call.method.span()), CalleeRow::Method(Name::from(&call.method)));
    }

    pub(super) fn path_called(&mut self, call: &ExprCall) {
        match Syntax::expr_path(&call.func) {
            Some(path) => self.path_call_lowered(path),
            None => (),
        }
    }

    pub(super) fn macro_called(&mut self, invoked: &Macro, position: Position) {
        let line = LineNumber::from(invoked.path.span());
        self.called(line, CalleeRow::Macro(MacroCall::new(UsePath::from(&invoked.path).written(), position)));
        self.expanded(invoked);
    }

    pub(super) fn expanded(&mut self, invoked: &Macro) {
        match StdNames::expansion(invoked) {
            Expansion::Statement => self.expressions_visited(Invocation { invoked }.arguments()),
            Expansion::Items => self.items_visited(Invocation { invoked }.items()),
        }
    }

    pub(super) fn expressions_visited(&mut self, arguments: Vec<Expr>) {
        arguments.iter().for_each(|argument| self.expansion_visited(|lowering| lowering.visit_expr(argument)));
    }

    pub(super) fn items_visited(&mut self, items: Vec<Item>) {
        items.iter().for_each(|item| self.expansion_visited(|lowering| lowering.visit_item(item)));
    }

    pub(super) fn path_call_lowered(&mut self, path: &ExprPath) {
        let segments = &path.path.segments;
        let line = segments
            .last()
            .map(|last| LineNumber::from(last.ident.span()))
            .unwrap_or_else(|| LineNumber::from(path.span()));
        let prefix = syn::Path {
            leading_colon: path.path.leading_colon,
            segments: segments.iter().take(segments.len().saturating_sub(1)).cloned().collect(),
        };
        let owner = self.owner_lowered(&prefix, line);
        self.called(line, CalleeRow::Path(UsePath::from(&path.path).written(), owner));
    }

    pub(super) fn owner_lowered(&self, prefix: &syn::Path, line: LineNumber) -> TypeRef<Written> {
        match prefix.segments.first() {
            Some(_) => self.generics.path_lowered(prefix, line),
            None => TypeRef::Opaque,
        }
    }

    pub(super) fn called(&mut self, line: LineNumber, callee: CalleeRow) {
        let located = self.here.located(line);
        let row = CallRow { located, callee, region: self.context.region(), subject: self.context.subject() };
        self.lowered.calls.push(row);
    }
}
