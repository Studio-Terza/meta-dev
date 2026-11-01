use lint_core::Name;
use lint_core::TypeName;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImplKind {
    InherentOnVertex,
    InherentOnValue,
    TraitOnVertex(Name),
    TraitOnValue(Name),
}

impl ImplKind {
    pub fn trait_name(&self) -> Option<&Name> {
        match self {
            ImplKind::TraitOnVertex(name) | ImplKind::TraitOnValue(name) => Some(name),
            ImplKind::InherentOnVertex | ImplKind::InherentOnValue => None,
        }
    }

    pub fn edge_source(&self, self_type: &TypeName, first_vertex_parameter: Option<&TypeName>) -> Option<TypeName> {
        match self {
            ImplKind::InherentOnVertex | ImplKind::TraitOnVertex(_) => Some(self_type.clone()),
            ImplKind::TraitOnValue(_) => first_vertex_parameter.cloned(),
            ImplKind::InherentOnValue => None,
        }
    }
}
