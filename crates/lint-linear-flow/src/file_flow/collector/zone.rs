use syn::ItemMod;
use syn::spanned::Spanned;

use lint_core::{Site, SourceFile};
use lint_layout::LineNumber;

use super::super::adapters::Adapters;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Zone {
    Boundary,
    Elsewhere,
}

impl Zone {
    pub fn of_file<'file>(file: &'file SourceFile, adapters: &Adapters<'file>) -> Zone {
        Zone::judged(adapters.evidence(file.path(), LineNumber::FIRST, LineNumber::from(usize::MAX)))
    }

    pub fn of_module<'file>(file: &'file SourceFile, module: &ItemMod, adapters: &Adapters<'file>) -> Zone {
        let span = module.span();
        Zone::judged(adapters.evidence(
            file.path(),
            LineNumber::from(span.start().line),
            LineNumber::from(span.end().line),
        ))
    }

    fn judged(evidence: Option<&Site<'_>>) -> Zone {
        match evidence {
            Some(_) => Zone::Boundary,
            None => Zone::Elsewhere,
        }
    }
}
