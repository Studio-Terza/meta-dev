use std::fmt;

use algebra::Semigroup;
use body_shape::TokenCount;
use equivalence::{Equivalence, Erased, Group, Holes};
use lint_core::{Fix, Site, Violation};

use crate::kind::Kind;
use crate::occurrence::Occurrence;

pub(crate) struct Finding<'sites> {
    size: TokenCount,
    kind: Kind,
    template: Template,
    others: &'sites [&'sites Site<'sites>],
}

impl Finding<'_> {
    pub(crate) fn violation<'file>(
        group: &Group<Occurrence<'file>>,
        equivalence: Equivalence,
    ) -> Option<Violation<'file>> {
        let sites = group.sites();
        let kind = group.members().iter().map(Occurrence::kind).reduce(Kind::combine).unwrap_or(Kind::Mixed);
        let spelled: Vec<&[_]> = group.members().iter().map(|member| member.literals().as_ref()).collect();
        let template = Template { holes: Holes::between(&spelled), naming: Naming::erased_by(equivalence) };
        sites.first().map(|site| {
            Violation::new(
                **site,
                Finding {
                    size: TokenCount::from(group.key()),
                    kind,
                    template,
                    others: sites.get(1..).unwrap_or_default(),
                },
            )
        })
    }
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let others: Vec<String> = self.others.iter().map(ToString::to_string).collect();
        write!(
            formatter,
            "{} tokens in {} {} copies, also at {}, {}",
            self.size,
            self.others.len() + 1,
            self.kind,
            others.join(", "),
            self.template
        )
    }
}

impl lint_core::Finding for Finding<'_> {
    fn fix(&self) -> Fix {
        self.kind.fix()
    }
}

#[derive(strum::Display, Clone, Copy, Debug)]
enum Naming {
    #[strum(serialize = "local names")]
    Local,
    #[strum(serialize = "names")]
    Every,
}

impl Naming {
    fn erased_by(equivalence: Equivalence) -> Naming {
        equivalence.unerased(Erased::Names, Naming::Local).unwrap_or(Naming::Every)
    }
}

#[derive(Clone, Copy, Debug)]
struct Template {
    holes: Holes,
    naming: Naming,
}

impl fmt::Display for Template {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.holes.differing() {
            Some(differing) => write!(formatter, "differing literals: {differing}"),
            None => write!(formatter, "identical up to {}", self.naming),
        }
    }
}
