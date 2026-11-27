use std::marker::PhantomData;

use crate::{Failed, Fixture};

#[derive(Debug)]
pub struct Rows<Row>(PhantomData<Row>);

impl<Row: Fixture> Rows<Row> {
    pub fn read() -> Result<Vec<Row>, Failed> {
        Ok(Row::STORY.rows::<Row>()?.into_values().collect())
    }
}
