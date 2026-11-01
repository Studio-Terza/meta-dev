mod block;
mod kind;

use std::collections::BTreeSet;
use std::iter;
use std::path::Path;

use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Ident, Macro, Token, Visibility, braced};

use lint_core::{Site, TypeName, Violation};
use lint_layout::LineNumber;

use crate::edge_scan::Edge;
use crate::finding::Finding;
use block::Block;
use kind::Kind;

pub struct Graph(Vec<Block>);

impl Graph {
    pub fn checked(invocation: &Macro, path: &Path) -> Vec<Violation> {
        invocation.parse_body::<Graph>().map_or_else(
            |_| {
                let site = Site::new(path, LineNumber::from(invocation.bang_token.span));
                vec![Violation::new(site, Finding::UnreadableGraph)]
            },
            |graph| graph.violations(path),
        )
    }

    pub fn violations(&self, path: &Path) -> Vec<Violation> {
        let edges: Vec<Edge> = self.0.iter().flat_map(Block::edges).collect();
        let reversed: Vec<Edge> = edges.iter().map(Edge::reversed).collect();
        let reachable = Self::closed(&self.states(Kind::Start), &edges);
        let ending = Self::closed(&self.states(Kind::Terminal), &reversed);
        self.0.iter().flat_map(|block| block.violations(path, &reachable, &ending)).collect()
    }

    fn states(&self, kind: Kind) -> BTreeSet<TypeName> {
        self.0.iter().filter_map(|block| block.state_of(kind)).collect()
    }

    fn closed(seen: &BTreeSet<TypeName>, edges: &[Edge]) -> BTreeSet<TypeName> {
        let grown: BTreeSet<TypeName> =
            edges.iter().filter_map(|edge| edge.target_from(seen)).chain(seen.iter().cloned()).collect();
        if grown.len() == seen.len() { grown } else { Self::closed(&grown, edges) }
    }
}

impl Parse for Graph {
    fn parse(input: ParseStream) -> syn::Result<Graph> {
        Attribute::parse_outer(input)?;
        input.parse::<Visibility>()?;
        input.parse::<Token![mod]>()?;
        input.parse::<Ident>()?;
        let content;
        braced!(content in input);
        iter::from_fn(|| (!content.is_empty()).then(|| content.parse())).collect::<syn::Result<_>>().map(Graph)
    }
}
