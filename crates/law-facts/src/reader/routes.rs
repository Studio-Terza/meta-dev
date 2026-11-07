mod closure;
mod generator;

use syn::punctuated::Punctuated;
use syn::{Expr, Macro, Path, QSelf, Token};

use crate::kind::Kind;
use crate::law_path::{LawPath, Owner, Route, Side};
use crate::reader::scope::Scope;
use crate::traits::Traits;
use closure::Closure;
use generator::Generator;
use lint_layout::Name;
use lint_placement::CrateName;

#[derive(Debug)]
pub(crate) struct Within<'scope> {
    scope: &'scope Scope,
    traits: &'scope Traits<'scope>,
    home: &'scope CrateName,
}

impl<'scope> Within<'scope> {
    pub(crate) const fn new(scope: &'scope Scope, traits: &'scope Traits<'scope>, home: &'scope CrateName) -> Self {
        Within { scope, traits, home }
    }

    fn owner(&self, written: &[Name], operand: Option<&Owner>) -> Option<Owner> {
        let places = self.scope.places(written);
        written.last().and_then(|last| self.scope.named(last)).and_then(|named| {
            Some(named)
                .filter(|named| self.traits.declaring(&places, named, self.home).is_none())
                .map(Owner::Named)
                .or_else(|| operand.cloned())
        })
    }

    fn operand(side: Side, generated: Option<&Name>) -> Option<Owner> {
        match side {
            Side::Input => generated.cloned().map(Owner::Named),
            Side::Output => Some(Owner::Operand),
            Side::Edge => Some(generated.cloned().map_or(Owner::Operand, Owner::Named)),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Routes<'scope> {
    within: &'scope Within<'scope>,
    operand: Option<Owner>,
}

impl<'scope> Routes<'scope> {
    const GENERATOR: usize = 1;
    const CHECKED_FROM: usize = 2;

    pub(crate) fn read(invocation: &Macro, within: &'scope Within<'scope>, kind: Kind) -> Vec<Route> {
        let arguments = invocation.parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated).unwrap_or_default();
        let generated =
            arguments.iter().nth(Self::GENERATOR).and_then(|found| Generator::new(within.scope).operand(found));
        arguments
            .iter()
            .skip(Self::CHECKED_FROM)
            .zip(Side::checked_by(kind))
            .filter_map(|(argument, side)| {
                Routes { within, operand: Within::operand(*side, generated.as_ref()) }.checked(*side, argument)
            })
            .collect()
    }

    pub(crate) const fn scope(&self) -> &Scope {
        self.within.scope
    }

    pub(crate) const fn operand(&self) -> Option<&Owner> {
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
        let owner = qself.map_or_else(
            || self.owner_segment(path),
            |qualified| self.within.scope.type_named(&qualified.ty).map(Owner::Named),
        );
        owner.map(|owner| LawPath::new(owner, member))
    }

    fn checked(&self, side: Side, argument: &Expr) -> Option<Route> {
        match argument {
            Expr::Path(found) => self.path(found.qself.as_ref(), &found.path).map(|path| Route::new(side, Some(path))),
            Expr::Call(call) => self.called(&call.func).map(|path| Route::new(side, Some(path))),
            Expr::Closure(closure) => Some(Route::new(side, Closure::resolved(self, closure))),
            Expr::Reference(reference) => self.checked(side, &reference.expr),
            Expr::Paren(inner) => self.checked(side, &inner.expr),
            _ => None,
        }
    }

    fn owner_segment(&self, path: &Path) -> Option<Owner> {
        let written: Vec<Name> = path.segments.iter().map(|segment| Name::from(&segment.ident)).collect();
        written.split_last().and_then(|(_, before)| self.within.owner(before, self.operand.as_ref()))
    }
}
