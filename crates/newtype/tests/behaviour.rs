use std::collections::BTreeSet;

use invariants::{Ignoring, ignored};
use newtype::{Checked, CheckedDeserialize, Counter, Deserialize, Display, Newtype, Serialize, Wrapping};
use newtype_derive::Arbitrary;
use proptest::collection::vec;
use proptest::prelude::{any, prop_assert, prop_assert_eq, proptest};
use proptest::strategy::{Strategy, ValueTree};
use proptest::test_runner::{Config, RngSeed, TestError, TestRunner};

ignored! {
    enum Ignored {
        Unserialised => "a value that fails to write or read as JSON is None, which the law compares",
        Refused => "a text the check or parse refuses is the None the law expects",
    }
}

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
        Some(number).filter(|candidate| **candidate < 1000).ok_or(Empty)
    }
}

#[derive(Newtype, Arbitrary, Clone, Debug, PartialEq, Eq)]
#[newtype(checked, strategy = "[0-9a-f]{40}")]
struct Digest(String);

impl Checked for Digest {
    type Error = Empty;

    fn checked(text: &String) -> Result<&String, Empty> {
        Some(text).filter(|candidate| candidate.len() == 40).ok_or(Empty)
    }
}

#[derive(Newtype, Arbitrary, Clone, Debug, PartialEq, Eq)]
#[newtype(strategy = Strategies::even())]
struct Even(u32);

struct Strategies;

impl Strategies {
    fn even() -> impl proptest::strategy::Strategy<Value = u32> {
        proptest::strategy::Strategy::prop_map(0u32..100, |half| half * 2)
    }
}

#[derive(Newtype, Arbitrary, Clone, Debug, PartialEq, Eq)]
struct Many<Item>(Vec<Item>);

#[derive(Newtype, Display)]
struct Shown(u32);

#[derive(Newtype, Display)]
struct Tagged<Tag>(Tag);

#[derive(Arbitrary, Clone, Debug, PartialEq, Eq)]
enum Shape<Mark> {
    Dot,
    Line(Mark),
    Box { wide: u8, tall: u16 },
}

impl<Mark> Shape<Mark> {
    fn kind(&self) -> &'static str {
        match self {
            Shape::Dot => "dot",
            Shape::Line(_) => "line",
            Shape::Box { .. } => "box",
        }
    }
}

#[derive(Arbitrary, Clone, Copy, Debug, PartialEq, Eq)]
#[newtype(from = Span::ordered)]
struct Span {
    start: u8,
    end: u8,
}

impl Span {
    fn ordered(one: u8, other: u8) -> Span {
        Span { start: one.min(other), end: one.max(other) }
    }
}

#[derive(Arbitrary, Clone, Copy, Debug, PartialEq, Eq)]
#[newtype(checked, from = Ratio::try_new)]
struct Ratio {
    above: u8,
    below: u8,
}

impl Ratio {
    fn try_new(above: u8, below: u8) -> Result<Ratio, Empty> {
        Some(Ratio { above, below }).filter(|ratio| ratio.below != 0).ok_or(Empty)
    }
}

#[derive(Arbitrary, Clone, Copy, Debug, PartialEq, Eq)]
#[newtype(generate = Parity::sampled)]
enum Parity {
    Odd(u32),
    Even(u32),
}

impl Parity {
    fn sampled() -> impl proptest::strategy::Strategy<Value = Parity> {
        proptest::strategy::Strategy::prop_map(any::<u32>(), Parity::from)
    }

    fn number(self) -> u32 {
        match self {
            Parity::Odd(number) | Parity::Even(number) => number,
        }
    }
}

impl From<u32> for Parity {
    fn from(number: u32) -> Parity {
        Some(number).filter(|candidate| candidate.is_multiple_of(2)).map_or(Parity::Odd(number), Parity::Even)
    }
}

#[derive(Arbitrary, Debug)]
#[newtype(checked, from = Refused::try_new)]
struct Refused;

impl Refused {
    fn try_new() -> Result<Refused, Empty> {
        Err(Empty)
    }
}

struct Pairs;

struct Json;

impl Json {
    fn written<Written: newtype::serde::Serialize>(value: &Written) -> Option<String> {
        serde_json::to_string(value).ignoring(Ignored::Unserialised)
    }

    fn read<Read: newtype::serde::de::DeserializeOwned>(text: Option<&str>) -> Option<Read> {
        text.and_then(|json| serde_json::from_str(json).ignoring(Ignored::Unserialised))
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

proptest! {
    #![proptest_config(Config { failure_persistence: None, rng_seed: RngSeed::Fixed(0), ..Config::default() })]
    #[test]
    fn a_tuple_newtype_returns_its_inner_value(number in any::<usize>()) {
        prop_assert_eq!(Rounds::new(number).into_inner(), number);
    }

    #[test]
    fn a_named_field_newtype_returns_its_inner_value(text in any::<String>()) {
        prop_assert_eq!(Label::new(text.clone()).into_inner(), text);
    }

    #[test]
    fn a_generic_newtype_returns_its_inner_value(number in any::<u8>()) {
        prop_assert_eq!(Wrapper::new(number).into_inner(), number);
    }

    #[test]
    fn a_derived_validated_arbitrary_only_builds_what_the_check_admits(word in any::<Word>()) {
        prop_assert!(Word::checked(&word.into_inner()).is_ok());
    }

    #[test]
    fn an_unchecked_newtype_reads_back_what_it_writes(rounds in any::<Rounds>()) {
        prop_assert_eq!(Json::read::<Rounds>(Json::written(&rounds).as_deref()), Some(rounds));
    }

    #[test]
    fn a_generic_newtype_reads_back_what_it_writes(wrapper in any::<Wrapper<u16>>()) {
        prop_assert_eq!(Json::read::<Wrapper<u16>>(Json::written(&wrapper).as_deref()), Some(wrapper));
    }

    #[test]
    fn a_checked_newtype_reads_back_what_it_writes(word in any::<Word>()) {
        prop_assert_eq!(Json::read::<Word>(Json::written(&word).as_deref()), Some(word));
    }

    #[test]
    fn a_checked_newtype_reads_what_its_check_builds(text in any::<String>()) {
        prop_assert_eq!(Json::word(&text), Word::try_new(text).ignoring(Ignored::Refused));
    }

    #[test]
    fn a_checked_newtype_built_by_try_new_reads_back_through_serde(text in any::<String>()) {
        let built = Word::try_new(text).ignoring(Ignored::Refused);
        prop_assert_eq!(built.as_ref().and_then(|word| Json::read::<Word>(Json::written(word).as_deref())), built);
    }

    #[test]
    fn a_newtype_writes_its_inner_value(number in any::<usize>()) {
        prop_assert_eq!(Json::written(&Rounds::new(number)), Json::written(&number));
    }

    #[test]
    fn a_count_reads_the_inner_value(rounds in any::<Rounds>()) {
        prop_assert_eq!(rounds.count(), rounds.into_inner());
    }

    #[test]
    fn an_increment_adds_one_and_stops_at_the_largest(rounds in any::<Rounds>()) {
        prop_assert_eq!(rounds.incremented().count(), rounds.count().saturating_add(1));
    }

    #[test]
    fn plus_adds_and_stops_at_the_largest(pair in any::<(Rounds, Rounds)>()) {
        prop_assert_eq!(pair.0.plus(pair.1).count(), pair.0.count().saturating_add(pair.1.count()));
    }

    #[test]
    fn a_total_is_the_sum_of_the_counts(numbers in vec(0usize..4096, 0..8)) {
        prop_assert_eq!(Rounds::total(numbers.iter().copied().map(Rounds::new)).count(), numbers.iter().sum::<usize>());
    }

    #[test]
    fn a_narrow_check_with_its_own_strategy_generates(small in any::<Small>()) {
        prop_assert!(Small::checked(&small.into_inner()).is_ok());
    }

    #[test]
    fn a_formatted_string_with_its_own_strategy_generates(digest in any::<Digest>()) {
        prop_assert!(Digest::checked(&digest.into_inner()).is_ok());
    }

    #[test]
    fn a_plain_newtype_takes_its_own_strategy(even in any::<Even>()) {
        prop_assert!(even.into_inner().is_multiple_of(2));
    }

    #[test]
    fn a_generic_field_that_is_not_the_bare_parameter_generates(many in any::<Many<u8>>()) {
        prop_assert_eq!(Many::new(many.clone().into_inner()), many);
    }

    #[test]
    fn a_derived_display_prints_the_inner_value(number in any::<u32>()) {
        prop_assert_eq!(Shown::new(number).to_string().parse::<u32>().ignoring(Ignored::Refused), Some(number));
    }

    #[test]
    fn a_generic_display_follows_its_parameter(tag in any::<char>()) {
        prop_assert_eq!(Tagged::new(tag).to_string(), tag.to_string());
    }

    #[test]
    fn a_constructor_shapes_every_generated_value(span in any::<Span>()) {
        prop_assert!(span.start <= span.end);
    }

    #[test]
    fn a_fallible_constructor_generates_only_what_it_accepts(ratio in any::<Ratio>()) {
        prop_assert_eq!(Ratio::try_new(ratio.above, ratio.below), Ok(ratio));
    }

    #[test]
    fn a_named_generator_replaces_the_derived_one(parity in any::<Parity>()) {
        prop_assert_eq!(Parity::from(parity.number()), parity);
    }
}

#[test]
fn a_check_decides_what_is_built() {
    assert_eq!(Json::built(&"a"), Ok(String::from("a")));
    assert_eq!(Json::built(&""), Err(Empty));
}

#[test]
fn a_checked_newtype_refuses_what_its_check_refuses() {
    assert_eq!(Json::word(&String::new()), None);
}

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

#[test]
fn derived_checked_generators_stay_under_the_rejection_bound() {
    let bounded = || {
        let config = Config { failure_persistence: None, rng_seed: RngSeed::Fixed(0), ..Config::default() };
        TestRunner::new(Config { max_local_rejects: config.cases, ..config })
    };
    assert!(bounded().run(&any::<Word>(), |_| Ok(())).is_ok());
    assert!(bounded().run(&any::<Small>(), |_| Ok(())).is_ok());
    assert!(bounded().run(&any::<Digest>(), |_| Ok(())).is_ok());
}

#[test]
fn derived_constructors_build_constants() {
    assert_eq!((Pairs::THREE.count(), Pairs::NONE.count()), (3, 0));
}

#[test]
fn an_enum_generates_every_variant() {
    let mut runner = TestRunner::deterministic();
    let kinds: BTreeSet<&str> =
        (0..64_u8).flat_map(|_| any::<Shape<char>>().new_tree(&mut runner)).map(|tree| tree.current().kind()).collect();
    assert_eq!(kinds, BTreeSet::from(["box", "dot", "line"]));
}

#[test]
fn a_constructor_that_refuses_everything_aborts_generation() {
    let refused = TestRunner::deterministic().run(&any::<Refused>(), |_| Ok(()));
    assert!(matches!(refused, Err(TestError::Abort(_))));
}
