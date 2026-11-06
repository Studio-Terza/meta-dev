use std::fmt;

use design_store::{Hit, Scope, SearchSegments};
use grammar::search::Terms;
use store_ports::Store;
use transcript_ports::{Authorship, Body, Instant, Uuid};

use crate::cli::failure::Failure;

#[derive(Debug)]
pub struct Search(Vec<Hit>);

impl Search {
    pub fn found(store: &impl Store, terms: Terms, scope: Scope) -> Result<Search, Failure> {
        Ok(Search(store.read(SearchSegments::matching(terms, scope))?))
    }
}

impl fmt::Display for Search {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|hit| {
            let uuid: &Uuid = hit.as_ref();
            let author: &Authorship = hit.as_ref();
            let stamp: &Instant = hit.as_ref();
            writeln!(formatter, "{uuid} {author} {stamp} {:?}", Body::from(hit).to_string())
        })
    }
}
