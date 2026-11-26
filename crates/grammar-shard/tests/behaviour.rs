#![deny(clippy::wildcard_enum_match_arm)]

use grammar::{Ignoring, Unlexed};
use grammar_shard::Sharding;
use proptest::prelude::{Strategy, prop_assert_eq, proptest};
use proptest::test_runner::{Config, RngSeed};

proptest! {
    #![proptest_config(Config { failure_persistence: None, rng_seed: RngSeed::Fixed(0), ..Config::default() })]
    #[test]
    fn a_sharding_is_displayed_and_parsed_back(
        (index, count) in (1..=u16::MAX).prop_flat_map(|count| (0..count).prop_map(move |index| (index, count)))
    ) {
        let shown = format!("{index}/{count}");
        let read = shown.parse::<Sharding>().ignoring(Unlexed::NotThisGrammar).map(|sharding| sharding.to_string());
        prop_assert_eq!(read, Some(shown));
    }
}
