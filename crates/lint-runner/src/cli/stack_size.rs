use std::io;
use std::panic::resume_unwind;
use std::thread::{self, ScopedJoinHandle};

#[derive(Clone, Copy, Debug)]
pub struct StackSize(usize);

impl StackSize {
    const BYTES_PER_MEBIBYTE: usize = 1024 * 1024;
    const LINT_MEBIBYTES: usize = 256;

    pub const LINT: Self = Self(Self::LINT_MEBIBYTES * Self::BYTES_PER_MEBIBYTE);

    pub fn builder(self) -> thread::Builder {
        thread::Builder::new().stack_size(self.0)
    }

    pub fn ran<Ran: Send>(self, work: impl FnOnce() -> Ran + Send) -> io::Result<Ran> {
        let joined = thread::scope(|scope| self.builder().spawn_scoped(scope, work).map(ScopedJoinHandle::join));
        joined.map(|ran| ran.unwrap_or_else(|payload| resume_unwind(payload)))
    }
}
