use laws::proptest::collection::vec;
use laws::proptest::prelude::{any, prop_assert, prop_assert_eq};
use newtype::{Checked, CheckedDeserialize, Counter, Deserialize, Display, Newtype, Serialize, Wrapping};
use newtype_derive::Arbitrary;

#[derive(Newtype, Arbitrary, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
struct Rounds(usize);

#[derive(Newtype, Arbitrary, Clone, Debug, PartialEq, Eq)]
struct Label {
    text: String,
}

#[derive(Newtype, Arbitrary, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct Wrapper<Wrapped>(Wrapped);

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("the text is empty")]
struct Empty;

#[derive(Newtype, Arbitrary, Serialize, CheckedDeserialize, Clone, Debug, PartialEq, Eq)]
#[newtype(checked)]
struct Word(String);

impl Checked for Word {
    type Error = Empty;

    fn checked(text: &String) -> Result<&String, Empty> {
        text.chars().next().map(|_| text).ok_or(Empty)
    }
}

#[derive(Newtype, Arbitrary, Clone, Debug, PartialEq, Eq)]
#[newtype(checked, strategy = 0u128..1000)]
struct Small(u128);

impl Checked for Small {
    type Error = Empty;

    fn checked(number: &u128) -> Result<&u128, Empty> {
        Some(number).filter(|number| **number < 1000).ok_or(Empty)
    }
}

#[derive(Newtype, Arbitrary, Clone, Debug, PartialEq, Eq)]
#[newtype(checked, strategy = "[0-9a-f]{40}")]
struct Digest(String);

impl Checked for Digest {
    type Error = Empty;

    fn checked(text: &String) -> Result<&String, Empty> {
        Some(text).filter(|text| text.len() == 40).ok_or(Empty)
    }
}

#[derive(Newtype, Arbitrary, Clone, Debug, PartialEq, Eq)]
#[newtype(strategy = Strategies::even())]
struct Even(u32);

struct Strategies;

impl Strategies {
    fn even() -> impl laws::proptest::strategy::Strategy<Value = u32> {
        laws::proptest::strategy::Strategy::prop_map(0u32..100, |half| half * 2)
    }
}

#[derive(Newtype, Arbitrary, Clone, Debug, PartialEq, Eq)]
struct Many<Item>(Vec<Item>);

#[derive(Newtype, Display)]
struct Shown(u32);

#[derive(Newtype, Display)]
struct Tagged<Tag>(Tag);

struct Pairs;

struct Json;

impl Json {
    fn written<Written: newtype::serde::Serialize>(value: &Written) -> Option<String> {
        serde_json::to_string(value).ok()
    }

    fn read<Read: newtype::serde::de::DeserializeOwned>(text: Option<&str>) -> Option<Read> {
        text.and_then(|text| serde_json::from_str(text).ok())
    }

    fn word(text: &String) -> Option<Word> {
        Json::read(Json::written(text).as_deref())
    }

    fn built(text: &&str) -> Result<String, Empty> {
        Word::try_new((*text).to_owned()).map(Word::into_inner)
    }
}

impl Pairs {
    const LARGEST: Rounds = Rounds(usize::MAX);
    const THREE: Rounds = Rounds::new(3);
    const NONE: Rounds = Rounds::new(0);

    fn counts(rounds: &[Rounds]) -> Vec<usize> {
        rounds.iter().copied().map(Counter::count).collect()
    }
}

laws::roundtrip!(
    a_tuple_newtype_returns_its_inner_value,
    any::<usize>(),
    |number: &usize| Rounds::new(*number),
    |rounds: &Rounds| rounds.into_inner()
);

laws::roundtrip!(
    a_named_field_newtype_returns_its_inner_value,
    any::<String>(),
    |text: &String| Label::new(text.clone()),
    |label: &Label| label.clone().into_inner()
);

laws::roundtrip!(
    a_generic_newtype_returns_its_inner_value,
    any::<u8>(),
    |number: &u8| Wrapper::new(*number),
    |wrapper: &Wrapper<u8>| wrapper.clone().into_inner()
);

laws::table!(a_check_decides_what_is_built, Json::built, ["a" => Ok(String::from("a")), "" => Err(Empty)]);

laws::law!(a_derived_validated_arbitrary_only_builds_what_the_check_admits, any::<Word>(), word, {
    prop_assert!(Word::checked(&word.into_inner()).is_ok());
});

laws::roundtrip!(
    an_unchecked_newtype_reads_back_what_it_writes,
    any::<Rounds>(),
    Json::written,
    |text: &Option<String>| Json::read::<Rounds>(text.as_deref()),
    Some
);

laws::roundtrip!(
    a_generic_newtype_reads_back_what_it_writes,
    any::<Wrapper<u16>>(),
    Json::written,
    |text: &Option<String>| Json::read::<Wrapper<u16>>(text.as_deref()),
    Some
);

laws::roundtrip!(
    a_checked_newtype_reads_back_what_it_writes,
    any::<Word>(),
    Json::written,
    |text: &Option<String>| Json::read::<Word>(text.as_deref()),
    Some
);

laws::law!(a_checked_newtype_reads_what_its_check_builds, any::<String>(), text, {
    prop_assert_eq!(Json::word(&text), Word::try_new(text).ok());
});

laws::law!(a_checked_newtype_built_by_try_new_reads_back_through_serde, any::<String>(), text, {
    let built = Word::try_new(text).ok();
    prop_assert_eq!(built.as_ref().and_then(|word| Json::read::<Word>(Json::written(word).as_deref())), built);
});

laws::law!(a_newtype_writes_its_inner_value, any::<usize>(), number, {
    prop_assert_eq!(Json::written(&Rounds::new(number)), Json::written(&number));
});

#[test]
fn a_checked_newtype_refuses_what_its_check_refuses() {
    assert_eq!(Json::word(&String::new()), None);
}

laws::law!(a_count_reads_the_inner_value, any::<Rounds>(), rounds, {
    prop_assert_eq!(rounds.count(), rounds.into_inner());
});

laws::law!(an_increment_adds_one_and_stops_at_the_largest, any::<Rounds>(), rounds, {
    prop_assert_eq!(rounds.incremented().count(), rounds.count().saturating_add(1));
});

laws::law!(plus_adds_and_stops_at_the_largest, any::<(Rounds, Rounds)>(), pair, {
    prop_assert_eq!(pair.0.plus(pair.1).count(), pair.0.count().saturating_add(pair.1.count()));
});

laws::law!(a_total_is_the_sum_of_the_counts, vec(0usize..4096, 0..8), numbers, {
    prop_assert_eq!(Rounds::total(numbers.iter().copied().map(Rounds::new)).count(), numbers.iter().sum::<usize>());
});

#[test]
fn counters_saturate_at_the_largest() {
    let largest = Pairs::LARGEST;
    assert_eq!(Pairs::counts(&[largest.incremented(), largest.plus(largest)]), [usize::MAX, usize::MAX]);
}

#[test]
fn nothing_counted_is_zero() {
    assert_eq!(Rounds::zero(), Rounds::new(0));
    assert_eq!(Rounds::total(std::iter::empty()), Rounds::zero());
}

laws::law!(a_narrow_check_with_its_own_strategy_generates, any::<Small>(), small, {
    prop_assert!(Small::checked(&small.into_inner()).is_ok());
});

laws::law!(a_formatted_string_with_its_own_strategy_generates, any::<Digest>(), digest, {
    prop_assert!(Digest::checked(&digest.into_inner()).is_ok());
});

laws::law!(a_plain_newtype_takes_its_own_strategy, any::<Even>(), even, {
    prop_assert_eq!(even.into_inner() % 2, 0);
});

laws::law!(a_generic_field_that_is_not_the_bare_parameter_generates, any::<Many<u8>>(), many, {
    prop_assert_eq!(Many::new(many.clone().into_inner()), many);
});

laws::roundtrip!(
    a_derived_display_prints_the_inner_value,
    any::<u32>(),
    |number: &u32| Shown::new(*number).to_string(),
    |text: &String| text.parse::<u32>().ok(),
    Some
);

laws::law!(a_generic_display_follows_its_parameter, any::<char>(), tag, {
    prop_assert_eq!(Tagged::new(tag).to_string(), tag.to_string());
});

#[test]
fn derived_constructors_build_constants() {
    assert_eq!((Pairs::THREE.count(), Pairs::NONE.count()), (3, 0));
}
