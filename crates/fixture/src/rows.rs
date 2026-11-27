use std::marker::PhantomData;

use crate::boundary::Story;
use crate::{Failed, Fixture};

#[derive(Debug)]
pub struct Rows<Row>(PhantomData<Row>);

impl<Row: Fixture> Rows<Row> {
    pub fn read() -> Result<Vec<Row>, Failed> {
        Ok(Story::of::<Row>().rows::<Row>()?.into_values().collect())
    }
}
