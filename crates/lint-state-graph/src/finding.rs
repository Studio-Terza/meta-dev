use std::collections::BTreeMap;
use std::fmt;

use quote::ToTokens;
use syn::{Member, Visibility};

use super::edge_scan::{Edge, EdgeCount};
use lint_core::Fix;
use lint_core::Most;
use lint_core::Site;
use lint_core::TypeName;
use lint_layout::Name;

pub enum Finding<'file> {
    TupleHoldsVertex,
    FreeTransition { target: TypeName },
    EdgeFromNonVertex { target: TypeName, owner: TypeName },
    HiddenEdge { edge: Edge },
    MutatesVertex { vertex: TypeName },
    ViewReturnsVertex { target: TypeName },
    DuplicateEdge { edge: Edge, sites: BTreeMap<Name, Site<'file>> },
    CrowdedVertex { source: TypeName, edges: EdgeCount, most: Most },
    ExposedField { field: Member, owner: Name, visibility: Visibility },
    ForeignMatch { enum_name: Name },
    UnreadableGraph,
    UnreachableState { state: TypeName },
    NoTerminalAhead { state: TypeName },
    TerminalHasEdges { state: TypeName },
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
            Finding::ExposedField { field, owner, visibility } => write!(
                formatter,
                "field {} of {owner} is {}",
                field.to_token_stream(),
                visibility.to_token_stream().to_string().chars().filter(|letter| *letter != ' ').collect::<String>()
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
        Fix::new(match self {
            Finding::TupleHoldsVertex => "return the vertex",
            Finding::FreeTransition { .. } => "make it a method on its source",
            Finding::EdgeFromNonVertex { .. } => "declare the vertex or move the edge",
            Finding::HiddenEdge { .. } => "edges are pub",
            Finding::MutatesVertex { .. } => "transitions consume self",
            Finding::ViewReturnsVertex { .. } => "take self by value or return a reference",
            Finding::DuplicateEdge { .. } => "keep one of these methods and remove the others",
            Finding::CrowdedVertex { .. } => "split the state into states with fewer edges each",
            Finding::ExposedField { .. } => "struct fields are private",
            Finding::ForeignMatch { .. } => "dispatch through a trait",
            Finding::UnreadableGraph => "write each state as a start, state or terminal block of its targets",
            Finding::UnreachableState { .. } => "add an edge to it from a reachable state, or remove it",
            Finding::NoTerminalAhead { .. } => "add an edge from it toward a terminal state",
            Finding::TerminalHasEdges { .. } => "remove its edges, or declare it with state instead of terminal",
        })
    }
}
