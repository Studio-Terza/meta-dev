use std::string::FromUtf8Error;

use laws::proptest;
use laws::{idempotent, roundtrip, table};
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
