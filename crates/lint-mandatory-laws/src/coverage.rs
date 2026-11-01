mod kind;

use std::collections::BTreeSet;

use proc_macro2::TokenStream;
use proc_macro2::TokenTree;
use syn::Macro;
use syn::visit::{self, Visit};

use crate::need::Need;
use kind::Kind;
use lint_core::SourceFile;
use lint_layout::Name;

#[derive(Debug, Default)]
pub(crate) struct Coverage {
    laws: BTreeSet<Name>,
    roundtrips: BTreeSet<Name>,
}

impl Coverage {
    pub(crate) fn read_all(files: &[SourceFile]) -> Coverage {
        files.iter().fold(Coverage::default(), Coverage::read)
    }

    pub(crate) fn covers(&self, need: Need, name: &Name) -> bool {
        match need {
            Need::Roundtrip => self.roundtrips.contains(name),
            Need::Law => self.laws.contains(name),
        }
    }

    fn read(mut self, file: &SourceFile) -> Coverage {
        self.visit_file(file.syntax());
        file.test_modules().iter().fold(self, Coverage::read)
    }

    fn named(tokens: TokenStream) -> Vec<Name> {
        tokens
            .into_iter()
            .flat_map(|tree| match tree {
                TokenTree::Ident(ident) => vec![Name::from(&ident)],
                TokenTree::Group(group) => Self::named(group.stream()),
                _ => Vec::new(),
            })
            .collect()
    }
}

impl<'ast> Visit<'ast> for Coverage {
    fn visit_macro(&mut self, invocation: &'ast Macro) {
        let kind = Kind::of_macro(invocation);
        let names = Self::named(invocation.tokens.clone());
        self.roundtrips.extend(names.iter().filter(|_| kind == Kind::Roundtrip).cloned());
        self.laws.extend(names.into_iter().filter(|_| kind != Kind::Other));
        visit::visit_macro(self, invocation);
    }
}
