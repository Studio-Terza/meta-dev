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
    pub fn state_of(&self, kind: Kind) -> Option<TypeName> {
        (self.kind == kind).then(|| self.state.clone())
    }

    pub fn edges(&self) -> impl Iterator<Item = Edge> {
        self.transitions.iter().map(|transition| Edge::new(self.state.clone(), transition.target().clone()))
    }

    pub fn violations(
        &self,
        path: &Path,
        reachable: &BTreeSet<TypeName>,
        ending: &BTreeSet<TypeName>,
    ) -> Vec<Violation> {
        let state = || self.state.clone();
        let lost = (!reachable.contains(&self.state)).then(|| Finding::UnreachableState { state: state() });
        let stuck = (!ending.contains(&self.state)).then(|| Finding::NoTerminalAhead { state: state() });
        let open = (self.kind == Kind::Terminal && !self.transitions.is_empty())
            .then(|| Finding::TerminalHasEdges { state: state() });
        let site = Site::new(path, self.line);
        [lost, stuck, open].into_iter().flatten().map(|finding| Violation::new(site.clone(), finding)).collect()
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
