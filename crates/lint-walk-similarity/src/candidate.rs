mod agreement;

use std::fmt;

use lint_core::Fix;
use lint_core::Site;
use lint_layout::Name;

use crate::walk::Walk;
use agreement::Agreement;

#[derive(Debug)]
pub(crate) struct Candidate<'file> {
    first: Name,
    second: Name,
    site: Site<'file>,
    other: Site<'file>,
    agreement: Agreement,
}

impl<'file> Candidate<'file> {
    pub(crate) fn between(first: &Walk<'_, 'file>, second: &Walk<'_, 'file>) -> Option<Candidate<'file>> {
        let agreement = Agreement::between(first, second);
        agreement.near_identical().map(|_| Candidate {
            first: AsRef::<Name>::as_ref(first).clone(),
            second: AsRef::<Name>::as_ref(second).clone(),
            site: *AsRef::<Site<'file>>::as_ref(first),
            other: *AsRef::<Site<'file>>::as_ref(second),
            agreement,
        })
    }
}

impl<'file> AsRef<Site<'file>> for Candidate<'file> {
    fn as_ref(&self) -> &Site<'file> {
        &self.site
    }
}

impl fmt::Display for Candidate<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} and {} (at {}) share {} outgoing methods and trait impls",
            self.first, self.second, self.other, self.agreement
        )
    }
}

impl lint_core::Finding for Candidate<'_> {
    fn fix(&self) -> Fix {
        Fix::new("unify with a standard trait, else a generic over an existing trait, else a derive")
    }
}
