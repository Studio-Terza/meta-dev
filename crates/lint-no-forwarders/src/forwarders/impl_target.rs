use syn::{ItemImpl, PathSegment};

use super::Forwarders;
use lint_core::Syntax;

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

    pub fn walked_by(self, forwarders: &mut Forwarders<'_, 'ast>, item: &'ast ItemImpl) {
        match self {
            ImplTarget::Inherent(segment) => forwarders.targeted(segment, item),
            ImplTarget::Skipped => {}
        }
    }
}
