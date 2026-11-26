use std::ops::Range;
use std::path::{Path, PathBuf};

use newtype::proptest::collection::vec;
use newtype::proptest::sample::select;
use newtype::proptest::strategy::Strategy;
use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[newtype(strategy = Untested::sampled())]
pub struct Untested(Vec<PathBuf>);

impl Untested {
    const SAMPLED: [&'static str; 3] = ["a.rs", "src/b.rs", "crates/c/src/lib.rs"];
    const COUNT: Range<usize> = 0..5;

    #[must_use]
    pub fn holds<'file>(&self, file: &'file Path) -> Option<&'file Path> {
        self.0.iter().any(|untested| untested == file).then_some(file)
    }

    pub(crate) fn sampled() -> impl Strategy<Value = Vec<PathBuf>> {
        vec(Strategy::prop_map(select(&Self::SAMPLED[..]), PathBuf::from), Self::COUNT)
    }
}
