pub use proptest;

#[macro_export]
macro_rules! law {
    ($(#[$meta:meta])* $name:ident, $strategy:expr, $value:ident, $body:block) => {
        $crate::proptest::proptest! {
            #![proptest_config($crate::proptest::test_runner::Config {
                failure_persistence: None,
                rng_seed: $crate::proptest::test_runner::RngSeed::Fixed(0),
                ..$crate::proptest::test_runner::Config::default()
            })]
            $(#[$meta])*
            #[test]
            fn $name($value in $strategy) $body
        }
    };
}

#[macro_export]
macro_rules! roundtrip {
    ($(#[$meta:meta])* $name:ident, $strategy:expr, $encode:expr, $decode:expr $(,)?) => {
        $crate::roundtrip!($(#[$meta])* $name, $strategy, $encode, $decode, ::core::convert::identity);
    };
    ($(#[$meta:meta])* $name:ident, $strategy:expr, $encode:expr, $decode:expr, $wrap:expr $(,)?) => {
        $crate::law!($(#[$meta])* $name, $strategy, value, {
            let encoded = $encode(&value);
            let decoded = $decode(&encoded);
            $crate::proptest::prop_assert_eq!(decoded, $wrap(value), "the roundtrip law is broken");
        });
    };
}

#[macro_export]
macro_rules! idempotent {
    ($(#[$meta:meta])* $name:ident, $strategy:expr, $function:expr $(,)?) => {
        $crate::law!($(#[$meta])* $name, $strategy, value, {
            let once = $function(&value);
            let twice = $function(&once);
            $crate::proptest::prop_assert_eq!(twice, once, "the idempotence law is broken");
        });
    };
}

#[macro_export]
macro_rules! table {
    ($(#[$meta:meta])* $name:ident, $function:expr, [$($input:expr => $want:expr),+ $(,)?] $(,)?) => {
        $(#[$meta])*
        #[test]
        fn $name() {
            $(
                assert_eq!(
                    $function(&$input),
                    $want,
                    "the table row is broken: {} => {}",
                    stringify!($input),
                    stringify!($want)
                );
            )+
        }
    };
}
