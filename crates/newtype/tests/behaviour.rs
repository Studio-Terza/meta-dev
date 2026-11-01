use laws::proptest::collection::vec;
use laws::proptest::prelude::{any, prop_assert, prop_assert_eq};
use newtype::{Arbitrary, Counter, Newtype, Validated};

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

struct Pairs;

impl Pairs {
    const LARGEST: Rounds = Rounds(usize::MAX);

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
