mod finding;

use lint_core::{Site, Violation};

use crate::occurrence::Occurrence;
use body_shape::TokenCount;
use finding::Finding;

#[derive(Clone, Debug)]
pub(crate) struct Family {
    size: TokenCount,
    copies: Vec<Occurrence>,
}

impl Family {
    pub(crate) fn new(size: TokenCount, copies: Vec<Occurrence>) -> Family {
        Family { size, copies }
    }

    pub(crate) fn size(&self) -> TokenCount {
        self.size
    }

    pub(crate) fn copies(&self) -> &[Occurrence] {
        &self.copies
    }

    pub(crate) fn violation(&self) -> Violation {
        let mut sites: Vec<&Site> = self.copies.iter().map(Occurrence::as_ref).collect();
        sites.sort();
        let others: Vec<String> = sites.iter().skip(1).map(ToString::to_string).collect();
        sites.first().map_or_else(
            || unreachable!("a family has copies"),
            |site| Violation::new((*site).clone(), Finding::new(self.size, &others)),
        )
    }
}
