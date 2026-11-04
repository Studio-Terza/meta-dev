use std::fmt;

use body_shape::TokenCount;
use equivalence::Group;
use lint_core::{Fix, Site, Violation};

use crate::kind::Kind;
use crate::occurrence::Occurrence;

pub(crate) struct Finding<'sites> {
    size: TokenCount,
    kind: Kind,
    others: &'sites [&'sites Site<'sites>],
}

impl Finding<'_> {
    pub(crate) fn violation<'file>(group: &Group<Occurrence<'file>>) -> Option<Violation<'file>> {
        let sites = group.sites();
        let kind = group.members().iter().map(Occurrence::kind).reduce(Kind::joined).unwrap_or(Kind::Mixed);
        sites.first().map(|site| {
            Violation::new(
                **site,
                Finding { size: group.key().size(), kind, others: sites.get(1..).unwrap_or_default() },
            )
        })
    }
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let others: Vec<String> = self.others.iter().map(ToString::to_string).collect();
        write!(
            formatter,
            "{} tokens in {} {} copies, also at {}",
            self.size,
            self.others.len() + 1,
            self.kind,
            others.join(", ")
        )
    }
}

impl lint_core::Finding for Finding<'_> {
    fn fix(&self) -> Fix {
        self.kind.fix()
    }
}
