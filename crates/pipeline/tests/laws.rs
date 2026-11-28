use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;
use std::time::Duration;

use file_ports::Size;
use pipeline::Measure;
use port_core::Calls;
use proptest::prelude::*;
use proptest::test_runner::{Config, RngSeed, TestCaseError};
use store_ports::Written;

struct Laws;

impl Laws {
    fn held<Measured: Measure + Clone + PartialEq + Debug>(
        [first, second, third]: [Measured; 3],
    ) -> Result<(), TestCaseError> {
        prop_assert_eq!(&Measured::default().joined(first.clone()), &first);
        prop_assert_eq!(&first.clone().joined(Measured::default()), &first);
        let left = first.clone().joined(second.clone()).joined(third.clone());
        prop_assert_eq!(left, first.joined(second.joined(third)));
        Ok(())
    }
}

proptest! {
    #![proptest_config(Config { failure_persistence: None, rng_seed: RngSeed::Fixed(0), ..Config::default() })]
    #[test]
    fn every_arbitrary_measure_has_an_identity_and_joins_associatively(
        counts in any::<[usize; 3]>(),
        sizes in any::<[u64; 3]>(),
        spans in any::<[Duration; 3]>(),
        units in any::<[(); 3]>(),
        lists in any::<[Vec<u8>; 3]>(),
        sets in any::<[BTreeSet<u8>; 3]>(),
        maps in any::<[BTreeMap<u8, usize>; 3]>(),
        calls in any::<[Calls; 3]>(),
        written in any::<[Written; 3]>(),
        stored in any::<[Size; 3]>(),
    ) {
        Laws::held(counts)?;
        Laws::held(sizes)?;
        Laws::held(spans)?;
        Laws::held(units)?;
        Laws::held(lists)?;
        Laws::held(sets)?;
        Laws::held(maps)?;
        Laws::held(calls)?;
        Laws::held(written)?;
        Laws::held(stored)?;
    }
}
