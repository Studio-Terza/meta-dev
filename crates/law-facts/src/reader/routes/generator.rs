use syn::{Expr, ExprCall, GenericArgument, PathArguments};

use crate::reader::scope::Scope;
use lint_layout::Name;

#[derive(Debug)]
pub(crate) struct Generator<'scope>(&'scope Scope);

impl<'scope> Generator<'scope> {
    pub(crate) fn new(scope: &'scope Scope) -> Self {
        Generator(scope)
    }

    pub(crate) fn operand(&self, generator: &Expr) -> Option<Name> {
        match generator {
            Expr::Call(call) => self.call(call),
            Expr::MethodCall(call) => self.operand(&call.receiver),
            Expr::Paren(inner) => self.operand(&inner.expr),
            _ => None,
        }
    }

    fn call(&self, call: &ExprCall) -> Option<Name> {
        match &*call.func {
            Expr::Path(found) => found.path.segments.last().and_then(|last| self.turbofish(&last.arguments)),
            _ => None,
        }
    }

    fn turbofish(&self, arguments: &PathArguments) -> Option<Name> {
        match arguments {
            PathArguments::AngleBracketed(bracketed) => {
                bracketed.args.first().and_then(|argument| self.argument(argument))
            }
            PathArguments::None | PathArguments::Parenthesized(_) => None,
        }
    }

    fn argument(&self, argument: &GenericArgument) -> Option<Name> {
        match argument {
            GenericArgument::Type(found) => self.0.type_named(found),
            _ => None,
        }
    }
}
