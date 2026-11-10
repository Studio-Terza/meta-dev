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
