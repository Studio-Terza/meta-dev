mod transition;

use std::collections::BTreeSet;
use std::path::Path;

use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Ident, Token, braced};

use lint_core::{Site, TypeName, Violation};
use lint_layout::LineNumber;

use super::kind::Kind;
use crate::edge_scan::Edge;
use crate::finding::Finding;
use transition::Transition;

pub struct Block {
    kind: Kind,
    state: TypeName,
    line: LineNumber,
    transitions: Punctuated<Transition, Token![,]>,
}

impl Block {
    pub fn state_of(&self, kind: Kind) -> Option<&TypeName> {
        (self.kind == kind).then_some(&self.state)
    }

    pub fn edges(&self) -> impl Iterator<Item = Edge<'_>> {
        self.transitions.iter().map(|transition| Edge::new(&self.state, transition.target()))
    }

    pub fn violations<'file>(
        &self,
        path: &'file Path,
        reachable: &BTreeSet<&TypeName>,
        ending: &BTreeSet<&TypeName>,
    ) -> Vec<Violation<'file>> {
        let state = &self.state;
        let lost = (!reachable.contains(state)).then_some(Finding::UnreachableState { state });
        let stuck = (!ending.contains(state)).then_some(Finding::NoTerminalAhead { state });
        let open = (self.kind == Kind::Terminal && !self.transitions.is_empty())
            .then_some(Finding::TerminalHasEdges { state });
        let site = Site::new(path, self.line);
        [lost, stuck, open].into_iter().flatten().map(|finding| Violation::new(site, finding)).collect()
    }
}

impl Parse for Block {
    fn parse(input: ParseStream) -> syn::Result<Block> {
        let kind = input.parse()?;
        let state: Ident = input.parse()?;
        let content;
        braced!(content in input);
        Ok(Block {
            kind,
            state: TypeName::named(&state),
            line: LineNumber::from(state.span()),
            transitions: Punctuated::parse_terminated(&content)?,
        })
    }
}
