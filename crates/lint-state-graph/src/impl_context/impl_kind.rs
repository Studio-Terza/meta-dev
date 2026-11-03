use lint_core::TypeName;
use lint_layout::Name;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImplKind<'found> {
    InherentOnVertex(&'found TypeName),
    InherentOnValue,
    TraitOnVertex(Name, &'found TypeName),
    TraitOnValue(Name),
}

impl<'found> ImplKind<'found> {
    pub fn of_trait(name: Name, vertex: Option<&'found TypeName>) -> ImplKind<'found> {
        match vertex {
            Some(vertex) => ImplKind::TraitOnVertex(name, vertex),
            None => ImplKind::TraitOnValue(name),
        }
    }

    pub fn trait_name(&self) -> Option<&Name> {
        match self {
            ImplKind::TraitOnVertex(name, _) | ImplKind::TraitOnValue(name) => Some(name),
            ImplKind::InherentOnVertex(_) | ImplKind::InherentOnValue => None,
        }
    }

    pub fn edge_source(&self, first_vertex_parameter: Option<&&'found TypeName>) -> Option<&'found TypeName> {
        match self {
            ImplKind::InherentOnVertex(vertex) | ImplKind::TraitOnVertex(_, vertex) => Some(vertex),
            ImplKind::TraitOnValue(_) => first_vertex_parameter.copied(),
            ImplKind::InherentOnValue => None,
        }
    }
}
