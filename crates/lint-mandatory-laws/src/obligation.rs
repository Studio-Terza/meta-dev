use std::fmt;

use crate::need::Need;
use crate::subject::Subject;
use lint_core::Site;
use lint_core::Violation;

#[derive(Debug)]
pub(crate) struct Obligation {
    subject: Subject,
    site: Site,
    need: Need,
}

impl Obligation {
    pub(crate) fn new(subject: Subject, site: Site, need: Need) -> Obligation {
        Obligation { subject, site, need }
    }

    pub(crate) fn violation(self) -> Violation {
        Violation::new(self.site.clone(), self)
    }
}

impl fmt::Display for Obligation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "`{}` {}", self.subject.owner(), self.need)
    }
}
