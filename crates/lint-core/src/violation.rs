use std::fmt;

use crate::site::Site;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Violation {
    site: Site,
    message: String,
}

impl Violation {
    pub fn new(site: Site, finding: impl fmt::Display) -> Violation {
        Violation { site, message: finding.to_string() }
    }

    #[must_use]
    pub fn site(&self) -> &Site {
        &self.site
    }
}

impl fmt::Display for Violation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.site, self.message)
    }
}
