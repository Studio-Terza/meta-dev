mod boundary;

use port_core::Op;

pub(crate) use boundary::EndedRow;

#[derive(Clone, Copy, Debug, Op)]
pub struct EndedTasks;
