use crate::Execute;

pub type Statements<'run> = dyn Execute + 'run;
