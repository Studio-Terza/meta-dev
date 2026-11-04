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

#[macro_export]
macro_rules! associative {
    ($(#[$meta:meta])* $name:ident, $strategy:expr, $operation:expr $(,)?) => {
        $crate::law!($(#[$meta])* $name, ($strategy, $strategy, $strategy), triple, {
            let (first, second, third) = triple;
            let left = $operation($operation(first.clone(), second.clone()), third.clone());
            let right = $operation(first, $operation(second, third));
            $crate::proptest::prop_assert_eq!(left, right, "the associativity law is broken");
        });
    };
}

#[macro_export]
macro_rules! commutative {
    ($(#[$meta:meta])* $name:ident, $strategy:expr, $operation:expr $(,)?) => {
        $crate::law!($(#[$meta])* $name, ($strategy, $strategy), pair, {
            let (first, second) = pair;
            let forward = $operation(first.clone(), second.clone());
            let backward = $operation(second, first);
            $crate::proptest::prop_assert_eq!(forward, backward, "the commutativity law is broken");
        });
    };
}

#[macro_export]
macro_rules! identity {
    ($(#[$meta:meta])* $name:ident, $strategy:expr, $operation:expr, $unit:expr $(,)?) => {
        $crate::law!($(#[$meta])* $name, $strategy, value, {
            let on_right = $operation(value.clone(), $unit);
            let on_left = $operation($unit, value.clone());
            $crate::proptest::prop_assert_eq!(&on_right, &value, "the identity law is broken");
            $crate::proptest::prop_assert_eq!(on_left, value, "the identity law is broken");
        });
    };
}

#[macro_export]
macro_rules! inverse {
    ($(#[$meta:meta])* $name:ident, $strategy:expr, $operation:expr, $invert:expr, $unit:expr $(,)?) => {
        $crate::law!($(#[$meta])* $name, $strategy, value, {
            let cancelled = $operation(value.clone(), $invert(value));
            $crate::proptest::prop_assert_eq!(cancelled, $unit, "the inverse law is broken");
        });
    };
}

#[macro_export]
macro_rules! absorbing {
    ($(#[$meta:meta])* $name:ident, $strategy:expr, $operation:expr, $zero:expr $(,)?) => {
        $crate::law!($(#[$meta])* $name, $strategy, value, {
            let on_right = $operation(value.clone(), $zero);
            let on_left = $operation($zero, value);
            $crate::proptest::prop_assert_eq!(&on_right, &$zero, "the absorbing law is broken");
            $crate::proptest::prop_assert_eq!(on_left, $zero, "the absorbing law is broken");
        });
    };
}

#[macro_export]
macro_rules! distributive {
    ($(#[$meta:meta])* $name:ident, $strategy:expr, $times:expr, $plus:expr $(,)?) => {
        $crate::law!($(#[$meta])* $name, ($strategy, $strategy, $strategy), triple, {
            let (first, second, third) = triple;
            let joined = $times(first.clone(), $plus(second.clone(), third.clone()));
            let separate = $plus($times(first.clone(), second), $times(first, third));
            $crate::proptest::prop_assert_eq!(joined, separate, "the distributivity law is broken");
        });
    };
}
