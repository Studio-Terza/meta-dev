use syntax::{ItemImpl, PathSegment};

use super::Scope;
use lint_core::Syntax;
use lint_sweep::{Descend, Visit};

pub enum ImplTarget<'ast> {
    Inherent(&'ast PathSegment),
    Skipped,
}

impl<'ast> ImplTarget<'ast> {
    pub fn describe(item: &'ast ItemImpl) -> ImplTarget<'ast> {
        Syntax::inherent(item)
            .and_then(|inherent| Syntax::type_path(&inherent.self_ty))
            .and_then(|path| path.path.segments.last())
            .map_or(ImplTarget::Skipped, ImplTarget::Inherent)
    }

    pub fn walked_by(self, scope: Scope) -> Visit<Scope> {
        match self {
            ImplTarget::Inherent(segment) => Visit::descending(scope.targeted(segment)),
            ImplTarget::Skipped => Visit::new(scope, Descend::Over),
        }
    }
}
