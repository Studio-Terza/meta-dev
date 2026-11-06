use std::marker::PhantomData;

use crate::Op;

#[derive(Debug)]
pub struct Entry<O: Op>(PhantomData<O>);

impl<O: Op> Entry<O> {
    pub(crate) fn admitted() -> Entry<O> {
        Entry(PhantomData)
    }
}
