mod closure;
mod generator;

use syn::punctuated::Punctuated;
use syn::{Expr, Macro, Path, QSelf, Token};

use crate::law_path::{LawPath, Route};
use crate::reader::scope::Scope;
use crate::traits::Traits;
use closure::Closure;
use generator::Generator;
use lint_layout::Name;

#[derive(Debug)]
pub(crate) struct Routes<'scope> {
    scope: &'scope Scope,
    traits: &'scope Traits,
    operand: Option<Name>,
}

impl<'scope> Routes<'scope> {
    const GENERATOR: usize = 1;
    const CHECKED_FROM: usize = 2;

    pub(crate) fn read(invocation: &Macro, scope: &'scope Scope, traits: &'scope Traits) -> Vec<Route> {
        let arguments = invocation.parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated).unwrap_or_default();
        let operand = arguments.iter().nth(Self::GENERATOR).and_then(|found| Generator::new(scope).operand(found));
        let routes = Routes { scope, traits, operand };
        arguments.iter().skip(Self::CHECKED_FROM).filter_map(|argument| routes.checked(argument)).collect()
    }

    pub(crate) fn scope(&self) -> &Scope {
        self.scope
    }

    pub(crate) fn operand(&self) -> Option<&Name> {
        self.operand.as_ref()
    }

    pub(crate) fn called(&self, function: &Expr) -> Option<LawPath> {
        match function {
            Expr::Path(found) => self.path(found.qself.as_ref(), &found.path),
            _ => None,
        }
    }

    pub(crate) fn path(&self, qself: Option<&QSelf>, path: &Path) -> Option<LawPath> {
        let member = path.segments.last().map(|last| Name::from(&last.ident))?;
        let owner = qself.map_or_else(|| self.owner_segment(path), |qualified| self.scope.type_named(&qualified.ty));
        owner.map(|owner| LawPath::new(owner, member))
    }

    fn checked(&self, argument: &Expr) -> Option<Route> {
        match argument {
            Expr::Path(found) => self.path(found.qself.as_ref(), &found.path).map(Route::Path),
            Expr::Call(call) => self.called(&call.func).map(Route::Path),
            Expr::Closure(closure) => Some(Closure::resolved(self, closure).map_or(Route::Closure, Route::Path)),
            Expr::Reference(reference) => self.checked(&reference.expr),
            Expr::Paren(inner) => self.checked(&inner.expr),
            _ => None,
        }
    }

    fn owner_segment(&self, path: &Path) -> Option<Name> {
        path.segments
            .iter()
            .rev()
            .nth(1)
            .and_then(|written| self.scope.named(Name::from(&written.ident)))
            .and_then(|named| self.owner(named))
    }

    fn owner(&self, named: Name) -> Option<Name> {
        Some(named).filter(|named| self.traits.declaring(named).is_none()).or_else(|| self.operand.clone())
    }
}
