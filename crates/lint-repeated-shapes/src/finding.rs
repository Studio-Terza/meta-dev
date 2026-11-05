use std::fmt;

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
        let kind = group.members().iter().map(Occurrence::kind).reduce(Kind::joined).unwrap_or(Kind::Mixed);
        let spelled: Vec<&[_]> = group.members().iter().map(|member| member.literals().as_ref()).collect();
        let template = Template { holes: Holes::between(&spelled), naming: Naming::erased_by(equivalence) };
        sites.first().map(|site| {
            Violation::new(
                **site,
                Finding { size: group.key().size(), kind, template, others: sites.get(1..).unwrap_or_default() },
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

#[derive(Clone, Copy, Debug)]
enum Naming {
    Local,
    Every,
}

impl Naming {
    fn erased_by(equivalence: Equivalence) -> Naming {
        equivalence.erases(Erased::Names).map_or(Naming::Local, |_| Naming::Every)
    }
}

impl fmt::Display for Naming {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Naming::Local => write!(formatter, "local names"),
            Naming::Every => write!(formatter, "names"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Template {
    holes: Holes,
    naming: Naming,
}

impl fmt::Display for Template {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.holes.none() {
            Some(_) => write!(formatter, "identical up to {}", self.naming),
            None => write!(formatter, "differing literals: {}", self.holes),
        }
    }
}
