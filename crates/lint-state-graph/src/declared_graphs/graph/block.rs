mod transition;

use std::collections::BTreeSet;
use std::path::Path;

use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Ident, Token, braced};

use lint_core::{Site, TypeName, Violation};
use lint_layout::LineNumber;

use super::kind::Kind;
use crate::edge::Edge;
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
        let lost = reachable.get(state).map_or_else(|| Some(Finding::UnreachableState { state }), |_| None);
        let stuck = ending.get(state).map_or_else(|| Some(Finding::NoTerminalAhead { state }), |_| None);
        let open = self
            .transitions
            .first()
            .filter(|_| self.kind == Kind::Terminal)
            .map(|_| Finding::TerminalHasEdges { state });
        let site = Site::new(path, self.line);
        [lost, stuck, open].into_iter().flatten().map(|finding| Violation::new(site, finding)).collect()
    }
}

impl Parse for Block {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let kind = input.parse()?;
        let state: Ident = input.parse()?;
        let content;
        braced!(content in input);
        Ok(Self {
            kind,
            state: TypeName::named(&state),
            line: LineNumber::from(state.span()),
            transitions: Punctuated::parse_terminated(&content)?,
        })
    }
}
