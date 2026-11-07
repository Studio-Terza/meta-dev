mod impl_kind;

pub use impl_kind::ImplKind;

use crate::declarations::Vertex;
use syn::{ImplItemFn, ItemImpl, Visibility};

use super::declarations::Declarations;
use super::function_shape::FunctionShape;
use lint_core::TypeName;
use lint_layout::Name;

pub struct ImplContext<'found> {
    self_type: TypeName,
    kind: ImplKind<'found>,
}

impl<'found> ImplContext<'found> {
    pub fn describe(item: &ItemImpl, declarations: &'found Declarations<'_>) -> Self {
        let self_type = declarations.resolved(&TypeName::from_type(&item.self_ty)).clone();
        let trait_name = item
            .trait_
            .as_ref()
            .and_then(|(_, path, _)| path.segments.last())
            .map(|segment| Name::from(&segment.ident));
        let vertex = declarations.vertex_named(&self_type).map(Vertex::name);
        let kind = trait_name.map_or_else(
            || vertex.map_or(ImplKind::InherentOnValue, ImplKind::InherentOnVertex),
            |name| ImplKind::of_trait(name, vertex),
        );
        ImplContext { self_type, kind }
    }

    pub fn self_type(&self) -> &TypeName {
        &self.self_type
    }

    pub fn kind(&self) -> &ImplKind<'found> {
        &self.kind
    }

    pub fn public<'function>(&self, function: &'function ImplItemFn) -> Option<&'function ImplItemFn> {
        self.kind
            .trait_name()
            .map(|_| function)
            .or_else(|| matches!(function.vis, Visibility::Public(_)).then_some(function))
    }

    pub fn edge_name(&self, function: &ImplItemFn) -> Name {
        self.kind.trait_name().map_or_else(|| Name::from(&function.sig.ident), Name::clone)
    }

    pub fn edge_source(&self, shape: &FunctionShape<'found, '_>) -> Option<&'found TypeName> {
        self.kind.edge_source(shape.vertex_parameters().first())
    }
}
