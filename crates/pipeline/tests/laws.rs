use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

use pipeline::Measure;
use proptest::prelude::*;
use proptest::test_runner::{Config, RngSeed};

proptest! {
    #![proptest_config(Config { failure_persistence: None, rng_seed: RngSeed::Fixed(0), ..Config::default() })]
    #[test]
    fn every_std_measure_has_an_identity_and_joins_associatively(
        [count, more, most] in any::<[usize; 3]>(),
        [size, larger, largest] in any::<[u64; 3]>(),
        [span, longer, longest] in any::<[Duration; 3]>(),
        [list, appended, extended] in any::<[Vec<u8>; 3]>(),
        [set, joined, widest] in any::<[BTreeSet<u8>; 3]>(),
        [map, merged, fullest] in any::<[BTreeMap<u8, usize>; 3]>(),
    ) {
        prop_assert_eq!([usize::default().joined(count), count.joined(usize::default())], [count, count]);
        prop_assert_eq!(count.joined(more).joined(most), count.joined(more.joined(most)));
        prop_assert_eq!([u64::default().joined(size), size.joined(u64::default())], [size, size]);
        prop_assert_eq!(size.joined(larger).joined(largest), size.joined(larger.joined(largest)));
        prop_assert_eq!([Duration::default().joined(span), span.joined(Duration::default())], [span, span]);
        prop_assert_eq!(span.joined(longer).joined(longest), span.joined(longer.joined(longest)));
        prop_assert_eq!([Vec::default().joined(list.clone()), list.clone().joined(Vec::default())], [list.clone(), list.clone()]);
        prop_assert_eq!(list.clone().joined(appended.clone()).joined(extended.clone()), list.joined(appended.joined(extended)));
        prop_assert_eq!([BTreeSet::default().joined(set.clone()), set.clone().joined(BTreeSet::default())], [set.clone(), set.clone()]);
        prop_assert_eq!(set.clone().joined(joined.clone()).joined(widest.clone()), set.joined(joined.joined(widest)));
        prop_assert_eq!([BTreeMap::default().joined(map.clone()), map.clone().joined(BTreeMap::default())], [map.clone(), map.clone()]);
        prop_assert_eq!(map.clone().joined(merged.clone()).joined(fullest.clone()), map.joined(merged.joined(fullest)));
    }
}
