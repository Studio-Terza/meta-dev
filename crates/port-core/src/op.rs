use std::marker::PhantomData;

use crate::boundary::OpName;

pub trait Op {
    const NAME: OpName;
}

#[derive(Debug)]
pub struct Pass<'call, O> {
    call: PhantomData<fn(&'call ()) -> &'call ()>,
    op: PhantomData<O>,
}

impl<O> Pass<'_, O> {
    pub(crate) fn minted() -> Self {
        Pass { call: PhantomData, op: PhantomData }
    }
}
