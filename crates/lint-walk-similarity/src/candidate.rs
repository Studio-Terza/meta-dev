mod agreement;

use std::fmt;

use lint_core::Fix;
use lint_core::Site;
use lint_layout::Name;

use crate::walk::Walk;
use agreement::Agreement;

#[derive(Debug)]
pub(crate) struct Candidate {
    first: Name,
    second: Name,
    site: Site,
    other: Site,
    agreement: Agreement,
}

impl Candidate {
    pub(crate) fn between(first: &Walk<'_>, second: &Walk<'_>) -> Option<Candidate> {
        let agreement = Agreement::between(first, second);
        agreement.near_identical().map(|_| Candidate {
            first: AsRef::<Name>::as_ref(first).clone(),
            second: AsRef::<Name>::as_ref(second).clone(),
            site: AsRef::<Site>::as_ref(first).clone(),
            other: AsRef::<Site>::as_ref(second).clone(),
            agreement,
        })
    }
}

impl AsRef<Site> for Candidate {
    fn as_ref(&self) -> &Site {
        &self.site
    }
}

impl fmt::Display for Candidate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} and {} (at {}) share {} outgoing methods and trait impls",
            self.first, self.second, self.other, self.agreement
        )
    }
}

impl lint_core::Finding for Candidate {
    fn fix(&self) -> Fix {
        Fix::new("unify with a standard trait, else a generic over an existing trait, else a derive")
    }
}
