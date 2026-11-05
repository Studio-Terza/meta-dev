use std::num::NonZeroUsize;
use std::thread;

use crate::cli::mode::boundary::Shard;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Workers(usize);

impl Workers {
    const MOST_READERS: usize = 4;

    pub(crate) fn available() -> Workers {
        Workers(thread::available_parallelism().map_or(1, NonZeroUsize::get).min(Self::MOST_READERS))
    }

    pub(crate) fn helper_shards(self) -> impl Iterator<Item = Shard> {
        (1..self.0).map(move |first| Shard::new(first, self.0))
    }
}
