use std::string::FromUtf8Error;

use laws::proptest;
use laws::{
    absorbing, associative, commutative, distributive, idempotent, identity, inverse, preserves, roundtrip, table,
};
use proptest::collection::vec;
use proptest::prelude::any;

struct Sample;

impl Sample {
    fn bytes(text: &str) -> Vec<u8> {
        text.as_bytes().to_owned()
    }

    fn text(bytes: &[u8]) -> Result<String, FromUtf8Error> {
        String::from_utf8(bytes.to_owned())
    }

    fn reversed(numbers: &[u8]) -> Vec<u8> {
        numbers.iter().rev().copied().collect()
    }

    fn sorted(numbers: &[u8]) -> Vec<u8> {
        let mut sorted = numbers.to_owned();
        sorted.sort_unstable();
        sorted
    }

    fn joined(mut first: Vec<u8>, second: Vec<u8>) -> Vec<u8> {
        first.extend(second);
        first
    }

    fn extended(numbers: &[u8]) -> Vec<u8> {
        numbers.iter().copied().chain([0]).collect()
    }
}

roundtrip!(a_plain_decode_is_compared_with_the_value, vec(any::<u8>(), 0..32), Sample::reversed, Sample::reversed);
roundtrip!(a_fallible_decode_is_compared_with_the_wrapped_value, any::<String>(), Sample::bytes, Sample::text, Ok);
roundtrip!(closures_serve_as_the_functions, any::<u8>(), |number: &u8| number ^ 1, |number: &u8| number ^ 1);
idempotent!(a_stable_function_holds, vec(any::<u8>(), 0..32), Sample::sorted);
table!(each_row_is_compared, Sample::bytes, ["a" => [97], "" => []]);

roundtrip!(
    #[should_panic(expected = "the roundtrip law is broken")]
    a_lossy_roundtrip_fails,
    vec(any::<u8>(), 0..32),
    Sample::extended,
    Sample::extended
);
idempotent!(
    #[should_panic(expected = "the idempotence law is broken")]
    a_function_that_keeps_changing_fails,
    vec(any::<u8>(), 0..32),
    Sample::extended
);
table!(
    #[should_panic(expected = "the table row is broken: \"b\"")]
    a_wrong_row_fails_and_is_named,
    Sample::bytes,
    ["a" => [97], "b" => [0]]
);

associative!(wrapping_addition_regroups, any::<u8>(), u8::wrapping_add);
commutative!(wrapping_addition_reorders, any::<u8>(), u8::wrapping_add);
identity!(zero_is_the_unit_of_wrapping_addition, any::<u8>(), u8::wrapping_add, 0);
inverse!(negation_cancels_wrapping_addition, any::<u8>(), u8::wrapping_add, u8::wrapping_neg, 0);
absorbing!(zero_absorbs_wrapping_multiplication, any::<u8>(), u8::wrapping_mul, 0);
distributive!(multiplication_spreads_over_addition, any::<u8>(), u8::wrapping_mul, u8::wrapping_add);

associative!(
    #[should_panic(expected = "the associativity law is broken")]
    subtraction_does_not_regroup,
    any::<u8>(),
    u8::wrapping_sub
);
commutative!(
    #[should_panic(expected = "the commutativity law is broken")]
    subtraction_does_not_reorder,
    any::<u8>(),
    u8::wrapping_sub
);
identity!(
    #[should_panic(expected = "the identity law is broken")]
    one_is_not_the_unit_of_addition,
    any::<u8>(),
    u8::wrapping_add,
    1
);
inverse!(
    #[should_panic(expected = "the inverse law is broken")]
    keeping_the_value_does_not_cancel,
    any::<u8>(),
    u8::wrapping_add,
    |value: u8| value,
    0
);
absorbing!(
    #[should_panic(expected = "the absorbing law is broken")]
    zero_does_not_absorb_addition,
    any::<u8>(),
    u8::wrapping_add,
    0
);
distributive!(
    #[should_panic(expected = "the distributivity law is broken")]
    addition_does_not_spread_over_multiplication,
    any::<u8>(),
    u8::wrapping_add,
    u8::wrapping_mul
);

preserves!(
    length_turns_joining_into_addition,
    vec(any::<u8>(), 0..8),
    Sample::joined,
    usize::wrapping_add,
    |numbers: Vec<u8>| numbers.len()
);
preserves!(
    #[should_panic(expected = "the preservation law is broken")]
    length_does_not_turn_joining_into_multiplication,
    vec(any::<u8>(), 0..8),
    Sample::joined,
    usize::wrapping_mul,
    |numbers: Vec<u8>| numbers.len()
);

laws::law!(a_given_value_is_built_once_and_shared_by_every_case, given table = Sample::sorted(&[3, 1, 2]), any::<u8>(), probe, {
    proptest::prop_assert_eq!(table.binary_search(&probe).is_ok(), [1, 2, 3].contains(&probe));
});

table!(every_value_of_a_finite_domain_is_checked_once, [1u8, 2, 3], number, {
    assert_eq!(Sample::reversed(&Sample::reversed(&[number])), [number]);
});

table!(
    #[should_panic(expected = "the value 3 is checked")]
    a_failing_value_of_a_finite_domain_fails,
    [1u8, 2, 3],
    number,
    {
        assert!(number < 3, "the value {number} is checked");
    }
);

table!(a_row_may_fail_with_an_error, ["a", "b"], text, {
    Sample::text(&Sample::bytes(text))?;
});

table!(
    #[should_panic(expected = "the table row is broken: invalid utf-8")]
    a_row_failing_with_an_error_fails,
    [vec![0xff_u8]],
    bytes,
    {
        Sample::text(&bytes)?;
    }
);

laws::generates!(
    a_generator_that_rejects_a_quarter_generates,
    proptest::strategy::Strategy::prop_filter(any::<u8>(), "a multiple of four", |number| number % 4 != 0)
);

#[test]
fn a_generator_that_rejects_most_of_what_it_draws_fails() {
    let strict =
        proptest::strategy::Strategy::prop_filter(any::<u8>(), "not a multiple of four", |number| number % 4 == 0);
    assert!(matches!(laws::Generation::bounded(&strict), Err(proptest::test_runner::TestError::Abort(_))));
}

#[test]
fn a_repeatable_run_draws_the_same_values_and_writes_no_regression_files() {
    let drawn: Vec<u64> = (0..2_u8)
        .flat_map(|_| {
            let mut runner = proptest::test_runner::TestRunner::new(laws::Generation::repeatable());
            proptest::strategy::Strategy::new_tree(&any::<u64>(), &mut runner)
        })
        .map(|tree| proptest::strategy::ValueTree::current(&tree))
        .collect();
    assert_eq!(drawn.first(), drawn.last());
    assert_eq!(drawn.len(), 2);
    assert!(laws::Generation::repeatable().failure_persistence.is_none());
}
