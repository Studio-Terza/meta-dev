use rayon::ThreadPoolBuilder;

use crate::failure::Failure;

#[derive(Debug)]
pub struct Pool;

impl Pool {
    const STACK_BYTES: usize = 256 * 1024 * 1024;

    pub fn installed<Made: Send>(work: impl FnOnce() -> Made + Send) -> Result<Made, Failure> {
        let pool = ThreadPoolBuilder::new().stack_size(Pool::STACK_BYTES).build().map_err(Failure::Threads)?;
        Ok(pool.install(work))
    }
}
