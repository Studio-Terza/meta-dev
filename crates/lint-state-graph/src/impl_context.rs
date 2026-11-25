mod impl_kind;
mod written;

pub use impl_kind::ImplKind;
pub use written::WrittenImpl;

use super::function_shape::FunctionShape;
use crate::function_shape::WrittenFn;
use lint_core::TypeName;
use lint_layout::Name;

pub struct ImplContext<'found> {
    self_type: TypeName,
    kind: ImplKind<'found>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exposure {
    ThroughTrait,
    Declared,
}

impl<'found> ImplContext<'found> {
    pub fn self_type(&self) -> &TypeName {
        &self.self_type
    }

    pub fn kind(&self) -> &ImplKind<'found> {
        &self.kind
    }

    pub fn public(&self, function: &WrittenFn) -> Option<Exposure> {
        self.kind.trait_name().map(|_| Exposure::ThroughTrait).or(function.public().map(|()| Exposure::Declared))
    }

    pub fn edge_name(&self, function: &WrittenFn) -> Name {
        self.kind.trait_name().map_or_else(|| AsRef::<Name>::as_ref(function).clone(), Name::clone)
    }

    pub fn edge_source(&self, shape: &FunctionShape<'found, '_>) -> Option<&'found TypeName> {
        self.kind.edge_source(shape.vertex_parameters().first())
    }
}
