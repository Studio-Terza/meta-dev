#![deny(clippy::wildcard_enum_match_arm)]

use grammar::{Ignoring, Unlexed};
use grammar_shard::Sharding;
use proptest::prelude::{prop_assert_eq, proptest};
use proptest::test_runner::{Config, RngSeed};

proptest! {
    #![proptest_config(Config { failure_persistence: None, rng_seed: RngSeed::Fixed(0), ..Config::default() })]
    #[test]
    fn a_sharding_in_its_own_spelling_is_shown_back_as_written(text in "(0|[1-9][0-9]{0,3})/[1-9][0-9]{0,3}") {
        let read = text.parse::<Sharding>().ignoring(Unlexed::NotThisGrammar);
        prop_assert_eq!(read.map_or_else(|| text.clone(), |sharding| sharding.to_string()), text);
    }

    #[test]
    fn any_sharding_read_from_text_reads_back_from_its_rendering(text in "[0-9/]{0,12}") {
        let read = text.parse::<Sharding>().ignoring(Unlexed::NotThisGrammar);
        let again = read.and_then(|sharding| sharding.to_string().parse::<Sharding>().ignoring(Unlexed::NotThisGrammar));
        prop_assert_eq!(again, read);
    }
}
