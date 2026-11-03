mod agreement;

use std::fmt;

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
        let agreement = Agreement::counted(
            first.steps().intersection(second.steps()).count(),
            first.steps().union(second.steps()).count(),
        );
        agreement.is_near_identical().then(|| Candidate {
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
            "{} and {} (at {}) share {} outgoing methods and trait impls; unify with a standard trait, else a generic over an existing trait, else a derive",
            self.first, self.second, self.other, self.agreement
        )
    }
}
