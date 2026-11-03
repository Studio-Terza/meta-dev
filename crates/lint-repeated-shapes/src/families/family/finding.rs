use std::fmt;

use body_shape::TokenCount;
use lint_core::Fix;
use lint_core::Site;

use crate::kind::Kind;

pub(super) struct Finding<'sites> {
    size: TokenCount,
    kind: Kind,
    others: &'sites [&'sites Site<'sites>],
}

impl<'sites> Finding<'sites> {
    pub(super) fn new(size: TokenCount, kind: Kind, others: &'sites [&'sites Site<'sites>]) -> Finding<'sites> {
        Finding { size, kind, others }
    }
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} tokens in {} {} copies, also at ", self.size, self.others.len() + 1, self.kind)?;
        self.others.iter().enumerate().try_for_each(|(index, site)| match index {
            0 => write!(formatter, "{site}"),
            _ => write!(formatter, ", {site}"),
        })
    }
}

impl lint_core::Finding for Finding<'_> {
    fn fix(&self) -> Fix {
        self.kind.fix()
    }
}
