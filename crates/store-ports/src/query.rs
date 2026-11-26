use port_core::Op;

use crate::{Execute, StoreFailure};

pub trait Query: Op {
    type Rows;

    fn rows(self, statements: &mut impl Execute) -> Result<Self::Rows, StoreFailure>;
}
