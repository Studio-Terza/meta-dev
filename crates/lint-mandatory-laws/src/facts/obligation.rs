use std::fmt;

use crate::need::Need;
use lint_core::Site;
use lint_core::Violation;
use lint_layout::Name;

#[derive(Debug)]
pub(crate) struct Obligation {
    subject: Name,
    site: Site,
    need: Need,
}

impl Obligation {
    pub(crate) fn new(subject: Name, site: Site, need: Need) -> Obligation {
        Obligation { subject, site, need }
    }

    pub(crate) fn subject(&self) -> &Name {
        &self.subject
    }

    pub(crate) fn need(&self) -> Need {
        self.need
    }

    pub(crate) fn violation(self) -> Violation {
        Violation::new(self.site.clone(), self)
    }
}

impl fmt::Display for Obligation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "`{}` {}", self.subject, self.need.fix())
    }
}
