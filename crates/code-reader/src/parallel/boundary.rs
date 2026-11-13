use std::num::NonZeroUsize;
use std::panic;
use std::thread;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Parallel(usize);

impl Default for Parallel {
    fn default() -> Parallel {
        Parallel(thread::available_parallelism().map_or(1, NonZeroUsize::get))
    }
}

impl Parallel {
    pub(crate) fn mapped<Item: Sync, Mapped: Send>(
        self,
        items: &[Item],
        map: impl Fn(&Item) -> Mapped + Sync,
    ) -> Vec<Mapped> {
        let share = items.len().div_ceil(self.0).max(1);
        thread::scope(|scope| {
            let shares: Vec<thread::ScopedJoinHandle<'_, Vec<Mapped>>> =
                items.chunks(share).map(|chunk| scope.spawn(|| chunk.iter().map(&map).collect())).collect();
            shares.into_iter().flat_map(|done| done.join().unwrap_or_else(|failure| panic::resume_unwind(failure))).collect()
        })
    }
}
