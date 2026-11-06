use std::fmt::{self, Display};
use std::marker::PhantomData;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OpName(&'static str);

impl OpName {
    #[must_use]
    pub const fn new(path: &'static str) -> OpName {
        OpName(path)
    }
}

impl Display for OpName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

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
