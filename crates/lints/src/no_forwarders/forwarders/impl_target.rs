use syn::{ItemImpl, PathSegment, Type};

use super::Forwarders;

pub enum ImplTarget<'ast> {
    Inherent(&'ast PathSegment),
    Skipped,
}

impl<'ast> ImplTarget<'ast> {
    pub fn describe(item: &'ast ItemImpl) -> ImplTarget<'ast> {
        match (&item.trait_, &*item.self_ty) {
            (None, Type::Path(path)) => path.path.segments.last().map_or(ImplTarget::Skipped, ImplTarget::Inherent),
            _ => ImplTarget::Skipped,
        }
    }

    pub fn walked_by(self, forwarders: &mut Forwarders<'_>, item: &'ast ItemImpl) {
        match self {
            ImplTarget::Inherent(segment) => {
                forwarders.targeting(segment);
                syn::visit::visit_item_impl(forwarders, item);
            }
            ImplTarget::Skipped => {}
        }
    }
}
