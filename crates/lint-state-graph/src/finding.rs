use std::collections::BTreeMap;
use std::fmt;

use quote::ToTokens;
use syn::{Field, Index};

use super::edge::Edge;
use super::edge_count::EdgeCount;
use lint_core::Bound;
use lint_core::Fix;
use lint_core::Site;
use lint_core::TypeName;
use lint_layout::Name;

pub enum Finding<'found> {
    TupleHoldsVertex,
    FreeTransition { target: &'found TypeName },
    EdgeFromNonVertex { target: &'found TypeName, owner: &'found TypeName },
    HiddenEdge { edge: Edge<'found> },
    MutatesVertex { vertex: &'found TypeName },
    ViewReturnsVertex { target: &'found TypeName },
    DuplicateEdge { edge: &'found Edge<'found>, sites: &'found BTreeMap<Name, Site<'found>> },
    CrowdedVertex { source: &'found TypeName, edges: EdgeCount, most: Bound },
    ExposedField { field: &'found Field, index: Index, owner: Name },
    ForeignMatch { enum_name: &'found Name },
    UnreadableGraph,
    UnreachableState { state: &'found TypeName },
    NoTerminalAhead { state: &'found TypeName },
    TerminalHasEdges { state: &'found TypeName },
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Finding::TupleHoldsVertex => formatter.write_str("returns a tuple holding a vertex"),
            Finding::FreeTransition { target } => write!(formatter, "takes a vertex and returns {target}"),
            Finding::EdgeFromNonVertex { target, owner } => {
                write!(formatter, "returns vertex {target} from non-vertex {owner}")
            }
            Finding::HiddenEdge { edge } => write!(formatter, "is a hidden edge {edge}"),
            Finding::MutatesVertex { vertex } => write!(formatter, "takes &mut self on vertex {vertex}"),
            Finding::ViewReturnsVertex { target } => write!(formatter, "is a view returning vertex {target}"),
            Finding::DuplicateEdge { edge, sites } => write!(
                formatter,
                "{edge} has {} edges, at most one allowed: {}",
                sites.len(),
                sites.iter().map(|(name, site)| format!("{name} at {site}")).collect::<Vec<String>>().join(", ")
            ),
            Finding::CrowdedVertex { source, edges, most } => {
                write!(formatter, "{source} has {edges} edges leaving it, at most {most} allowed")
            }
            Finding::ExposedField { field, index, owner } => write!(
                formatter,
                "field {} of {owner} is {}",
                field.ident.as_ref().map_or_else(|| index.to_token_stream(), ToTokens::to_token_stream),
                field.vis.to_token_stream().to_string().chars().filter(|letter| *letter != ' ').collect::<String>()
            ),
            Finding::ForeignMatch { enum_name } => write!(formatter, "match on {enum_name} outside its file"),
            Finding::UnreadableGraph => formatter.write_str("states! graph cannot be read, so it cannot be checked"),
            Finding::UnreachableState { state } => {
                write!(formatter, "state {state} cannot be reached from a start state")
            }
            Finding::NoTerminalAhead { state } => write!(formatter, "state {state} cannot reach a terminal state"),
            Finding::TerminalHasEdges { state } => write!(formatter, "terminal state {state} has outgoing edges"),
        }
    }
}

impl lint_core::Finding for Finding<'_> {
    fn fix(&self) -> Fix {
        match self {
            Finding::TupleHoldsVertex => Fix::new("return the vertex"),
            Finding::FreeTransition { .. } => Fix::new("make it a method on its source"),
            Finding::EdgeFromNonVertex { .. } => Fix::new("declare the vertex or move the edge"),
            Finding::HiddenEdge { .. } => Fix::new("edges are pub"),
            Finding::MutatesVertex { .. } => Fix::new("transitions consume self"),
            Finding::ViewReturnsVertex { .. } => Fix::new("take self by value or return a reference"),
            Finding::DuplicateEdge { .. } => Fix::new("keep one of these methods and remove the others"),
            Finding::CrowdedVertex { .. } => Fix::new("split the state into states with fewer edges each"),
            Finding::ExposedField { .. } => Fix::new("struct fields are private"),
            Finding::ForeignMatch { .. } => Fix::new("dispatch through a trait"),
            Finding::UnreadableGraph => Fix::new("write each state as a start, state or terminal block of its targets"),
            Finding::UnreachableState { .. } => Fix::new("add an edge to it from a reachable state, or remove it"),
            Finding::NoTerminalAhead { .. } => Fix::new("add an edge from it toward a terminal state"),
            Finding::TerminalHasEdges { .. } => {
                Fix::new("remove its edges, or declare it with state instead of terminal")
            }
        }
    }
}
