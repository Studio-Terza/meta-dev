use laws::proptest::collection::vec;
use laws::proptest::prelude::{any, prop_assert, prop_assert_eq};
use newtype::{Counter, Display, Newtype, Validated};
use newtype_derive::Arbitrary;

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq)]
struct Rounds(usize);

#[derive(Newtype, Arbitrary, Clone, Debug, PartialEq, Eq)]
struct Label {
    text: String,
}

#[derive(Newtype, Arbitrary, Clone, Debug, PartialEq, Eq)]
struct Wrapper<Wrapped>(Wrapped);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Empty;

#[derive(Newtype, Arbitrary, Clone, Debug, PartialEq, Eq)]
#[newtype(check = Word::check, error = Empty)]
struct Word(String);

impl Word {
    fn check(text: &str) -> Result<(), Empty> {
        if text.is_empty() { Err(Empty) } else { Ok(()) }
    }

    fn checked(text: &&str) -> Result<String, Empty> {
        Word::try_new((*text).to_owned()).map(Validated::into_inner)
    }
}

#[derive(Newtype, Arbitrary, Clone, Debug, PartialEq, Eq)]
#[newtype(check = Small::check, error = Empty, strategy = 0u128..1000)]
struct Small(u128);

impl Small {
    fn check(number: &u128) -> Result<(), Empty> {
        if *number < 1000 { Ok(()) } else { Err(Empty) }
    }
}

#[derive(Newtype, Arbitrary, Clone, Debug, PartialEq, Eq)]
#[newtype(check = Digest::check, error = Empty, strategy = "[0-9a-f]{40}")]
struct Digest(String);

impl Digest {
    fn check(text: &str) -> Result<(), Empty> {
        if text.len() == 40 { Ok(()) } else { Err(Empty) }
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

impl Pairs {
    const LARGEST: Rounds = Rounds(usize::MAX);
    const THREE: Rounds = Rounds::new(3);
    const NONE: Rounds = Rounds::zero();

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

laws::table!(a_check_decides_what_is_built, Word::checked, ["a" => Ok(String::from("a")), "" => Err(Empty)]);

laws::law!(a_derived_plain_arbitrary_builds_through_new, any::<Wrapper<Label>>(), wrapper, {
    prop_assert_eq!(Wrapper::new(wrapper.clone().into_inner()), wrapper);
});

laws::law!(a_derived_validated_arbitrary_only_builds_what_the_check_admits, any::<Word>(), word, {
    prop_assert!(Word::check(&word.into_inner()).is_ok());
});

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
    let rounds: Vec<Rounds> = numbers.iter().copied().map(Rounds::new).collect();
    prop_assert_eq!(Rounds::total(rounds.into_iter()).count(), numbers.iter().sum::<usize>());
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
    prop_assert!(Small::check(&small.into_inner()).is_ok());
});

laws::law!(a_formatted_string_with_its_own_strategy_generates, any::<Digest>(), digest, {
    prop_assert!(Digest::check(&digest.into_inner()).is_ok());
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
