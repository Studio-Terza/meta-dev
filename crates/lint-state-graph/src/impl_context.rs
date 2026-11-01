mod impl_kind;

pub use impl_kind::ImplKind;

use syn::{ImplItemFn, ItemImpl, Visibility};

use super::declarations::Declarations;
use super::function_shape::FunctionShape;
use lint_core::Name;
use lint_core::TypeName;

pub struct ImplContext {
    self_type: TypeName,
    kind: ImplKind,
}

impl ImplContext {
    pub fn describe(item: &ItemImpl, declarations: &Declarations) -> ImplContext {
        let self_type = declarations.resolve(TypeName::from_type(&item.self_ty));
        let trait_name = item
            .trait_
            .as_ref()
            .and_then(|(_, path, _)| path.segments.last())
            .map(|segment| Name::from(&segment.ident));
        let kind = match (trait_name, declarations.is_vertex(&self_type)) {
            (Some(name), true) => ImplKind::TraitOnVertex(name),
            (Some(name), false) => ImplKind::TraitOnValue(name),
            (None, true) => ImplKind::InherentOnVertex,
            (None, false) => ImplKind::InherentOnValue,
        };
        ImplContext { self_type, kind }
    }

    pub fn self_type(&self) -> &TypeName {
        &self.self_type
    }

    pub fn in_trait(&self) -> bool {
        self.kind.trait_name().is_some()
    }

    pub fn kind(&self) -> &ImplKind {
        &self.kind
    }

    pub fn is_public(&self, function: &ImplItemFn) -> bool {
        self.in_trait() || matches!(function.vis, Visibility::Public(_))
    }

    pub fn edge_name(&self, function: &ImplItemFn) -> Name {
        self.kind.trait_name().map_or_else(|| Name::from(&function.sig.ident), Name::clone)
    }

    pub fn edge_source(&self, shape: &FunctionShape) -> Option<TypeName> {
        self.kind.edge_source(&self.self_type, shape.vertex_parameters().first())
    }
}
