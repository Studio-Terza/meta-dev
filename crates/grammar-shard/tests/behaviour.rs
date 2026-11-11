#![deny(clippy::wildcard_enum_match_arm)]

use grammar_shard::Sharding;
use laws::proptest::prelude::Strategy;

laws::roundtrip!(
    a_sharding_is_displayed_and_parsed_back,
    (1..=u16::MAX).prop_flat_map(|count| (0..count).prop_map(move |index| (index, count))),
    |(index, count): &(u16, u16)| format!("{index}/{count}"),
    |text: &String| text.parse::<Sharding>().ok().map(|sharding| sharding.to_string()),
    |(index, count): (u16, u16)| Some(format!("{index}/{count}"))
);
