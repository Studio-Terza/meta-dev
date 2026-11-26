use std::path::Path;
use std::thread::{self, Scope, ScopedJoinHandle};

use invariants::Rejoined;

#[derive(Clone, Copy, Debug)]
pub(super) struct Spread;

impl Spread {
    const THREADS: usize = 4;
    const SHARE: usize = 8;

    pub(super) fn mapped<Out: Send>(paths: &[impl AsRef<Path>], each: impl Fn(&Path) -> Out + Sync) -> Vec<Out> {
        let held: Vec<&Path> = paths.iter().map(AsRef::as_ref).collect();
        let share = held.len().div_ceil(Self::THREADS).max(Self::SHARE);
        thread::scope(|scope| {
            let mut parts = held.chunks(share);
            let first = parts.next();
            let spawned: Vec<ScopedJoinHandle<'_, Vec<Out>>> =
                parts.map(|part| Self::spawned(scope, part, &each)).collect();
            let mine = first.map_or_default(|part| Self::each(part, &each));
            mine.into_iter().chain(spawned.into_iter().flat_map(|handle| handle.join().rejoined())).collect()
        })
    }

    fn spawned<'scope, Out: Send + 'scope>(
        scope: &'scope Scope<'scope, '_>,
        part: &'scope [&Path],
        each: &'scope (impl Fn(&Path) -> Out + Sync),
    ) -> ScopedJoinHandle<'scope, Vec<Out>> {
        scope.spawn(|| Self::each(part, each))
    }

    fn each<Out>(part: &[&Path], each: &impl Fn(&Path) -> Out) -> Vec<Out> {
        part.iter().map(|path| each(path)).collect()
    }
}
