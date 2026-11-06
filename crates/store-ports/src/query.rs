use port_core::Op;

use crate::{Statements, StoreFailure};

pub trait Query: Op {
    type Rows;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Self::Rows, StoreFailure>;
}
