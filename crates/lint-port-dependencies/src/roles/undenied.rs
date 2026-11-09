use std::collections::BTreeSet;
use std::fmt;

use syn::punctuated::Punctuated;
use syn::{Path, Token};

use lint_core::{Finding, Fix, SourceFile};
use lint_layout::{Name, UsePath, Word, words};
use workspace_metadata::Role;

#[derive(Clone, Debug)]
pub(super) struct Undenied {
    role: Role,
    lint: UsePath,
}

impl Undenied {
    const DENIED: [[Word<'static>; 2]; 1] = [words!["clippy", "wildcard_enum_match_arm"]];
    const LEVELS: [&'static str; 2] = ["deny", "forbid"];

    pub(super) fn all(role: Role) -> Vec<Undenied> {
        Self::DENIED
            .iter()
            .map(|lint| Undenied { role, lint: lint.iter().map(|segment| Name::from(*segment)).collect() })
            .collect()
    }

    pub(super) fn missing(role: Role, root: &SourceFile) -> Vec<Undenied> {
        let denied: BTreeSet<UsePath> = root
            .syntax()
            .attrs
            .iter()
            .filter(|attribute| Self::LEVELS.iter().any(|level| attribute.path().is_ident(level)))
            .filter_map(|attribute| attribute.parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated).ok())
            .flatten()
            .map(|path| UsePath::from(&path))
            .collect();
        Self::all(role).into_iter().filter(|undenied| denied.get(&undenied.lint).is_none()).collect()
    }
}

impl fmt::Display for Undenied {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "serves the {} role and does not deny {} at its crate root, as the crate the lexers came from did",
            self.role, self.lint
        )
    }
}

impl Finding for Undenied {
    fn fix(&self) -> Fix {
        Fix::new("start the crate's lib.rs with #![deny(<lint>)] for each lint every crate of the role denies")
    }
}
