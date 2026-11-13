use std::num::NonZeroUsize;
use std::panic;
use std::sync::{Mutex, PoisonError};
use std::thread::{self, Scope, ScopedJoinHandle};

#[derive(Debug)]
struct Running<'scope, Mapped>(Vec<ScopedJoinHandle<'scope, Mapped>>);

impl<Mapped> Running<'_, Mapped> {
    fn joined(self) -> Vec<Mapped> {
        self.0.into_iter().map(|done| done.join().unwrap_or_else(|failure| panic::resume_unwind(failure))).collect()
    }
}

#[derive(Debug)]
struct Spawner<'scope, 'env, Work> {
    scope: &'scope Scope<'scope, 'env>,
    work: &'env Work,
}

impl<'scope, Work: Sync> Spawner<'scope, '_, Work> {
    fn spawned<Share: Send + 'scope, Mapped: Send + 'scope>(&self, share: Share) -> ScopedJoinHandle<'scope, Mapped>
    where
        Work: Fn(Share) -> Mapped,
    {
        let work = self.work;
        self.scope.spawn(move || work(share))
    }
}

#[derive(Debug)]
pub(crate) struct Parked<Item>(Mutex<Option<Item>>);

impl<Item> From<Item> for Parked<Item> {
    fn from(item: Item) -> Parked<Item> {
        Parked(Mutex::new(Some(item)))
    }
}

impl<Item> Parked<Item> {
    pub(crate) fn taken(&self) -> Option<Item> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner).take()
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Parallel(usize);

impl Parallel {
    const MOST_SHARES: usize = 4;

    pub(crate) fn available() -> Parallel {
        Parallel(thread::available_parallelism().map_or(1, NonZeroUsize::get).min(Self::MOST_SHARES))
    }

    pub(crate) fn mapped<Item: Sync, Mapped: Send>(
        self,
        items: &[Item],
        map: impl Fn(&Item) -> Mapped + Sync,
    ) -> Vec<Mapped> {
        let mapped = self.shares(items).worked(|share: &[Item]| share.iter().map(&map).collect::<Vec<Mapped>>());
        mapped.into_iter().flatten().collect()
    }

    pub(crate) fn shares<Item>(self, items: &[Item]) -> Shares<'_, Item> {
        Shares(items.chunks(items.len().div_ceil(self.0).max(1)).collect())
    }
}

#[derive(Debug)]
pub(crate) struct Shares<'items, Item>(Vec<&'items [Item]>);

impl<Item: Sync> Shares<'_, Item> {
    pub(crate) fn worked<Mapped: Send>(self, work: impl Fn(&[Item]) -> Mapped + Sync) -> Vec<Mapped> {
        let shares = self.0;
        thread::scope(|scope| {
            let spawner = Spawner { scope, work: &work };
            Running(shares.into_iter().map(|share| spawner.spawned(share)).collect()).joined()
        })
    }
}
