use std::io;

use crate::FileFailure;

#[derive(Clone, Debug)]
pub enum Recall<Value> {
    Found(Value),
    Failed(i32),
    Unrepeatable,
}

impl<Value: Clone> Recall<Value> {
    pub fn remembered(fresh: Result<Value, FileFailure>) -> Recall<Value> {
        match fresh {
            Ok(found) => Recall::Found(found),
            Err(failure) => io::Error::from(failure).raw_os_error().map_or(Recall::Unrepeatable, Recall::Failed),
        }
    }

    pub fn replayed(&self, again: impl FnOnce() -> Result<Value, FileFailure>) -> Result<Value, FileFailure> {
        match self {
            Recall::Found(found) => Ok(found.clone()),
            Recall::Failed(code) => Err(FileFailure::from(io::Error::from_raw_os_error(*code))),
            Recall::Unrepeatable => again(),
        }
    }
}
