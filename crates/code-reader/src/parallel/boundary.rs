use std::collections::BTreeMap;
use std::num::NonZeroUsize;
use std::panic;
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

#[derive(Clone, Copy, Debug)]
pub(crate) struct Parallel(usize);

impl Parallel {
    pub(crate) fn available() -> Parallel {
        Parallel(thread::available_parallelism().map_or(1, NonZeroUsize::get))
    }

    pub(crate) fn mapped<Item: Sync, Mapped: Send>(
        self,
        items: &[Item],
        map: impl Fn(&Item) -> Mapped + Sync,
    ) -> Vec<Mapped> {
        let shares: Vec<&[Item]> = items.chunks(self.share(items.len())).collect();
        let mapped = Self::joined(shares, |share: &[Item]| share.iter().map(&map).collect::<Vec<Mapped>>());
        mapped.into_iter().flatten().collect()
    }

    pub(crate) fn consumed<Item: Send, Mapped: Send>(
        self,
        items: Vec<Item>,
        map: impl Fn(Vec<Item>) -> Mapped + Sync,
    ) -> Vec<Mapped> {
        let share = self.share(items.len());
        let shares = items.into_iter().enumerate().fold(
            BTreeMap::new(),
            |mut shares: BTreeMap<usize, Vec<Item>>, (at, item)| {
                shares.entry(at / share).or_default().push(item);
                shares
            },
        );
        Self::joined(shares.into_values().collect(), map)
    }

    fn share(self, count: usize) -> usize {
        count.div_ceil(self.0).max(1)
    }

    fn joined<Share: Send, Mapped: Send>(shares: Vec<Share>, work: impl Fn(Share) -> Mapped + Sync) -> Vec<Mapped> {
        thread::scope(|scope| {
            let spawner = Spawner { scope, work: &work };
            Running(shares.into_iter().map(|share| spawner.spawned(share)).collect()).joined()
        })
    }
}
