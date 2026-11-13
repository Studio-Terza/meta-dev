mod queries;

use std::collections::BTreeSet;

use code_model::Model;
use lint_core::{Headline, ModelLint, Report, Site, Violation};

use queries::{Queries, Unformatted};

#[derive(Debug)]
pub struct StoreQueries;

impl ModelLint for StoreQueries {
    fn report(model: &Model) -> Report<'_> {
        let queries = Queries::new(model);
        let flagged: BTreeSet<Site<'_>> = model.quotations().filter_map(|placed| queries.unformatted(placed)).collect();
        Report::new(
            Headline::new(
                "every store_ports::query! writes its SQL as sqlformat formats it, starting at column 0 inside r#\"…\"#",
            ),
            flagged.into_iter().map(|site| Violation::new(site, Unformatted)).collect(),
        )
    }
}
