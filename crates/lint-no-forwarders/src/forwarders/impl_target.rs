use syntax::{ItemImpl, PathSegment};

use super::Target;
use super::struct_fields::StructFields;
use lint_core::Syntax;
use lint_layout::Name;

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

    pub fn target(self) -> Option<Target> {
        match self {
            ImplTarget::Inherent(segment) => Some(Target {
                self_type: Name::from(&segment.ident),
                self_ident: Some(Name::from(&segment.ident)),
                concrete: StructFields::type_arguments(segment),
            }),
            ImplTarget::Skipped => None,
        }
    }
}
