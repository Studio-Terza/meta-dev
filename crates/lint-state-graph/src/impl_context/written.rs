use syntax::ItemImpl;

use super::{ImplContext, ImplKind};
use crate::declarations::{Declarations, Vertex};
use crate::function_shape::{FunctionShape, WrittenFn};
use lint_core::TypeName;
use lint_layout::Name;

#[derive(Debug)]
pub struct WrittenImpl {
    self_type: TypeName,
    trait_name: Option<Name>,
    functions: Vec<WrittenFn>,
}

impl From<&ItemImpl> for WrittenImpl {
    fn from(item: &ItemImpl) -> WrittenImpl {
        WrittenImpl {
            self_type: TypeName::from_type(&item.self_ty),
            trait_name: item
                .trait_
                .as_ref()
                .and_then(|(_, path, _)| path.segments.last())
                .map(|segment| Name::from(&segment.ident)),
            functions: item.items.iter().filter_map(FunctionShape::function).map(WrittenFn::from).collect(),
        }
    }
}

impl WrittenImpl {
    pub fn functions(&self) -> &[WrittenFn] {
        &self.functions
    }
}

impl<'found> ImplContext<'found> {
    pub fn resolved(item: &WrittenImpl, declarations: &'found Declarations) -> ImplContext<'found> {
        let self_type = declarations.resolved(&item.self_type).clone();
        let vertex = declarations.vertex_named(&self_type).map(Vertex::name);
        let kind = item.trait_name.clone().map_or_else(
            || vertex.map_or(ImplKind::InherentOnValue, ImplKind::InherentOnVertex),
            |name| ImplKind::of_trait(name, vertex),
        );
        ImplContext { self_type, kind }
    }
}
