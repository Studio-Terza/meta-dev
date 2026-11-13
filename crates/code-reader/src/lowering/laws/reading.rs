mod closure;

use syn::punctuated::Punctuated;
use syn::{Expr, ExprCall, GenericArgument, Macro, Path, PathArguments, QSelf, Token, Type, TypePath};

use code_model::{LawKind, Side};
use lint_layout::Name;

use super::{LoweredOperation, LoweredOwner, LoweredRoute};
use crate::lowering::signature::InScope;
use closure::Closure;

#[derive(Clone, Copy, Debug)]
pub(super) struct Reading<'scope> {
    scope: &'scope InScope,
}

#[derive(Debug)]
pub(super) struct Routed {
    operation: Option<LoweredOperation>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Checking<'scope, 'owner> {
    scope: &'scope InScope,
    fallback: Option<&'owner LoweredOwner>,
}

#[derive(Clone, Copy, Debug)]
struct Generator<'scope> {
    scope: &'scope InScope,
}

impl<'scope> Reading<'scope> {
    const GENERATOR: usize = 1;
    const CHECKED_FROM: usize = 2;

    pub(super) fn new(scope: &'scope InScope) -> Reading<'scope> {
        Reading { scope }
    }

    pub(super) fn routes(self, invocation: &Macro, kind: LawKind) -> Vec<LoweredRoute> {
        let arguments = invocation.parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated).unwrap_or_default();
        let generated =
            arguments.iter().nth(Self::GENERATOR).and_then(|found| Generator { scope: self.scope }.operand(found));
        arguments
            .iter()
            .skip(Self::CHECKED_FROM)
            .zip(Side::checked_by(kind))
            .filter_map(|(argument, side)| {
                let fallback = LoweredOwner::of_side(*side, generated.as_ref());
                let checking = Checking { scope: self.scope, fallback: fallback.as_ref() };
                checking.checked(argument).map(|routed| LoweredRoute {
                    side: *side,
                    operation: routed.operation,
                    fallback: fallback.clone(),
                })
            })
            .collect()
    }
}

impl LoweredOwner {
    fn of_side(side: Side, generated: Option<&LoweredOwner>) -> Option<LoweredOwner> {
        match side {
            Side::Input => generated.cloned(),
            Side::Output => Some(LoweredOwner::Operand),
            Side::Edge => Some(generated.cloned().unwrap_or(LoweredOwner::Operand)),
        }
    }
}

impl<'scope, 'owner> Checking<'scope, 'owner> {
    pub(super) fn scope(self) -> &'scope InScope {
        self.scope
    }

    pub(super) fn fallback(self) -> Option<&'owner LoweredOwner> {
        self.fallback
    }

    pub(super) fn called(self, function: &Expr) -> Option<LoweredOperation> {
        match function {
            Expr::Path(found) => self.path(found.qself.as_ref(), &found.path),
            _ => None,
        }
    }

    fn checked(self, argument: &Expr) -> Option<Routed> {
        match argument {
            Expr::Path(found) => self.path(found.qself.as_ref(), &found.path).map(Routed::found),
            Expr::Call(call) => self.called(&call.func).map(Routed::found),
            Expr::Closure(closure) => Some(Routed { operation: Closure::resolved(self, closure) }),
            Expr::Reference(reference) => self.checked(&reference.expr),
            Expr::Paren(inner) => self.checked(&inner.expr),
            _ => None,
        }
    }

    fn path(self, qself: Option<&QSelf>, path: &Path) -> Option<LoweredOperation> {
        let member = path.segments.last().map(|last| Name::from(&last.ident))?;
        let owner = qself.map_or_else(
            || self.prefix(path),
            |qualified| Some(LoweredOwner::Typed(self.scope.lowered(&qualified.ty))),
        )?;
        Some(LoweredOperation { owner, member })
    }

    fn prefix(self, path: &Path) -> Option<LoweredOwner> {
        let mut prefix = path.clone();
        prefix.segments.pop();
        prefix.segments.pop_punct();
        prefix.segments.first()?;
        Some(LoweredOwner::Typed(self.scope.lowered(&Type::Path(TypePath { qself: None, path: prefix }))))
    }
}

impl Routed {
    fn found(operation: LoweredOperation) -> Routed {
        Routed { operation: Some(operation) }
    }
}

impl Generator<'_> {
    fn operand(self, generator: &Expr) -> Option<LoweredOwner> {
        match generator {
            Expr::Call(call) => self.call(call),
            Expr::MethodCall(call) => self.operand(&call.receiver),
            Expr::Paren(inner) => self.operand(&inner.expr),
            _ => None,
        }
    }

    fn call(self, call: &ExprCall) -> Option<LoweredOwner> {
        match &*call.func {
            Expr::Path(found) => found.path.segments.last().and_then(|last| self.turbofish(&last.arguments)),
            _ => None,
        }
    }

    fn turbofish(self, arguments: &PathArguments) -> Option<LoweredOwner> {
        match arguments {
            PathArguments::AngleBracketed(bracketed) => {
                bracketed.args.first().and_then(|argument| self.argument(argument))
            }
            PathArguments::None | PathArguments::Parenthesized(_) => None,
        }
    }

    fn argument(self, argument: &GenericArgument) -> Option<LoweredOwner> {
        match argument {
            GenericArgument::Type(found) => Some(LoweredOwner::Typed(self.scope.lowered(found))),
            _ => None,
        }
    }
}
