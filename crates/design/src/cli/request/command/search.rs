use design_model::Segment;
use design_store::{Hit, Scope, SearchSegments};
use grammar::search::Terms;
use store_ports::Store;

use crate::cli::failure::Failure;
use crate::cli::output::Line;
use crate::cli::request::command::rendering::Refs;

#[derive(Debug)]
pub struct Search;

impl Search {
    pub fn found(store: &impl Store, terms: Terms, scope: Scope) -> Result<Vec<Line>, Failure> {
        let hits: Vec<Hit> = store.read(SearchSegments::matching(terms, scope))?;
        let refs = Refs::read(store, hits.iter().map(Segment::from))?;
        Ok(refs.hits(&hits))
    }
}
