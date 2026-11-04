use std::collections::BTreeMap;

use syn::visit::{self, Visit};
use syn::{Block, Expr, ExprCall, ExprMethodCall, FnArg, Item, Signature, Type};

use lint_core::Syntax;
use lint_layout::{Name, Whole, Word};

use crate::callee::{Callee, Order, Receiver};

#[derive(Debug, Default)]
pub(super) struct Calls {
    bound: BTreeMap<Name, Name>,
    made: Vec<Callee>,
}

impl Calls {
    const OWNER: Word<'static> = Word::new("self");

    pub(super) fn made(signature: &Signature, body: &Block) -> Vec<Callee> {
        let bound = signature.inputs.iter().filter_map(Self::binding).fold(BTreeMap::new(), |mut bound, binding| {
            bound.insert(binding.name, binding.type_name);
            bound
        });
        let mut scan = Calls { bound, made: Vec::new() };
        scan.visit_block(body);
        scan.made
    }

    fn binding(input: &FnArg) -> Option<Binding> {
        Syntax::typed(input).and_then(|typed| {
            let name = Syntax::pat_ident(&typed.pat).map(|ident| Name::from(&ident.ident));
            name.zip(Self::type_name(&typed.ty)).map(|(name, type_name)| Binding { name, type_name })
        })
    }

    fn type_name(found: &Type) -> Option<Name> {
        match found {
            Type::Reference(reference) => Self::type_name(&reference.elem),
            _ => Name::of_type(found),
        }
    }

    fn receiver(&self, found: &Expr, called: Option<Order>) -> Receiver {
        match found {
            Expr::Path(path) => path.path.get_ident().map_or(Receiver::Unknown, |ident| self.named(&Name::from(ident))),
            Expr::MethodCall(_) | Expr::Call(_) => called.map_or(Receiver::Unknown, Receiver::Call),
            _ => Receiver::Unknown,
        }
    }

    fn named(&self, name: &Name) -> Receiver {
        name.matches::<Whole>(&[Self::OWNER]).map_or_else(
            || self.bound.get(name).map_or(Receiver::Unknown, |owner| Receiver::Bound(owner.clone())),
            |_| Receiver::Owner,
        )
    }
}

struct Binding {
    name: Name,
    type_name: Name,
}

impl<'ast> Visit<'ast> for Calls {
    fn visit_item(&mut self, _nested: &'ast Item) {}

    fn visit_expr_method_call(&mut self, found: &'ast ExprMethodCall) {
        self.visit_expr(&found.receiver);
        let called = self.made.len().checked_sub(1).map(Order::new);
        let receiver = self.receiver(&found.receiver, called);
        found.args.iter().for_each(|argument| self.visit_expr(argument));
        self.made.push(Callee::Method { receiver, name: Name::from(&found.method) });
    }

    fn visit_expr_call(&mut self, found: &'ast ExprCall) {
        visit::visit_expr_call(self, found);
        let segments: Vec<Name> = Syntax::expr_path(&found.func)
            .map(|path| path.path.segments.iter().map(|segment| Name::from(&segment.ident)).collect())
            .unwrap_or_default();
        let owner = segments.iter().rev().nth(1).cloned();
        self.made.extend(segments.last().cloned().map(|name| Callee::Path { owner, name }));
    }
}
