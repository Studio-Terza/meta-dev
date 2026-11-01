use std::collections::BTreeMap;
use std::fmt;

use quote::ToTokens;
use syn::{Member, Visibility};

use super::edge_scan::{Edge, EdgeCount};
use lint_core::Most;
use lint_core::Site;
use lint_core::TypeName;
use lint_layout::Name;

pub enum Finding {
    TupleHoldsVertex,
    FreeTransition { target: TypeName },
    EdgeFromNonVertex { target: TypeName, owner: TypeName },
    HiddenEdge { edge: Edge },
    MutatesVertex { vertex: TypeName },
    ViewReturnsVertex { target: TypeName },
    DuplicateEdge { edge: Edge, sites: BTreeMap<Name, Site> },
    CrowdedVertex { source: TypeName, edges: EdgeCount, most: Most },
    ExposedField { field: Member, owner: Name, visibility: Visibility },
    ForeignMatch { enum_name: Name },
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Finding::TupleHoldsVertex => formatter.write_str("returns a tuple holding a vertex; return the vertex"),
            Finding::FreeTransition { target } => {
                write!(formatter, "takes a vertex and returns {target}; make it a method on its source")
            }
            Finding::EdgeFromNonVertex { target, owner } => write!(
                formatter,
                "returns vertex {target} from non-vertex {owner}; declare the vertex or move the edge"
            ),
            Finding::HiddenEdge { edge } => {
                write!(formatter, "is a hidden edge {edge}; edges are pub")
            }
            Finding::MutatesVertex { vertex } => {
                write!(formatter, "takes &mut self on vertex {vertex}; transitions consume self")
            }
            Finding::ViewReturnsVertex { target } => {
                write!(formatter, "is a view returning vertex {target}; take self by value or return a reference")
            }
            Finding::DuplicateEdge { edge, sites } => write!(
                formatter,
                "{edge} has {} edges, at most one allowed: {}",
                sites.len(),
                sites.iter().map(|(name, site)| format!("{name} at {site}")).collect::<Vec<String>>().join(", ")
            ),
            Finding::CrowdedVertex { source, edges, most } => {
                write!(formatter, "{source} has {edges} edges leaving it, at most {most} allowed")
            }
            Finding::ExposedField { field, owner, visibility } => write!(
                formatter,
                "field {} of {owner} is {}; struct fields are private",
                field.to_token_stream(),
                visibility.to_token_stream().to_string().chars().filter(|letter| *letter != ' ').collect::<String>()
            ),
            Finding::ForeignMatch { enum_name } => {
                write!(formatter, "match on {enum_name} outside its file; dispatch through a trait")
            }
        }
    }
}
