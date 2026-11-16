pub use proptest;

#[macro_export]
macro_rules! law {
    ($(#[$meta:meta])* $name:ident, given $given:ident = $setup:expr, $strategy:expr, $value:ident, $body:block) => {
        $(#[$meta])*
        #[test]
        fn $name() -> Result<(), Box<dyn ::std::error::Error>> {
            let $given = $setup;
            let mut runner = $crate::proptest::test_runner::TestRunner::new($crate::proptest::test_runner::Config {
                failure_persistence: None,
                rng_seed: $crate::proptest::test_runner::RngSeed::Fixed(0),
                ..$crate::proptest::test_runner::Config::default()
            });
            runner.run(&$strategy, |$value| {
                let _: () = $body;
                Ok(())
            })?;
            Ok(())
        }
    };
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
    ($(#[$meta:meta])* $name:ident, $domain:expr, $row:ident, $body:block $(,)?) => {
        $(#[$meta])*
        #[test]
        fn $name() {
            ::core::iter::IntoIterator::into_iter($domain)
                .try_for_each(|$row| -> Result<(), Box<dyn ::std::error::Error>> {
                    let _: () = $body;
                    Ok(())
                })
                .unwrap_or_else(|failure| panic!("the table row is broken: {failure}"));
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

#[macro_export]
macro_rules! preserves {
    ($(#[$meta:meta])* $name:ident, $strategy:expr, $source:expr, $target:expr, $edge:expr $(,)?) => {
        $crate::law!($(#[$meta])* $name, ($strategy, $strategy), pair, {
            let (first, second) = pair;
            let mapped = $edge($source(first.clone(), second.clone()));
            let combined = $target($edge(first), $edge(second));
            $crate::proptest::prop_assert_eq!(mapped, combined, "the preservation law is broken");
        });
    };
}

#[macro_export]
macro_rules! crud {
    ($(#[$meta:meta])* $name:ident, $record:ty, $fresh:expr, $opened:expr $(,)?) => {
        $crate::law!(
            $(#[$meta])* $name,
            (
                $fresh,
                $fresh,
                $crate::proptest::prelude::any::<$crate::proptest::sample::Index>(),
                $crate::proptest::prop_oneof![
                    $crate::proptest::strategy::Just(::store_ports::Argument::Null),
                    $crate::proptest::strategy::Strategy::prop_map($crate::proptest::prelude::any::<i64>(), ::store_ports::Argument::Integer),
                    $crate::proptest::strategy::Strategy::prop_map($crate::proptest::prelude::any::<String>(), ::store_ports::Argument::Text),
                    $crate::proptest::strategy::Strategy::prop_map($crate::proptest::prelude::any::<Vec<u8>>(), ::store_ports::Argument::Blob),
                ],
            ),
            drawn,
            {
            use ::store_ports::{Created, Record, Store};
            let (first, second, column, argument) = drawn;
            let run = ::port_core::Run::root();
            let store = run.port($opened);
            let made: $record = store
                .create(Created::<$record>::new(first.clone()))
                .map_err(|failure| $crate::proptest::test_runner::TestCaseError::fail(failure.to_string()))?;
            let read = |store: &::port_core::Port<'_, _>| {
                store
                    .read(<$record>::fetched(made.key()))
                    .map(|rows| rows.into_iter().next())
                    .map_err(|failure| $crate::proptest::test_runner::TestCaseError::fail(failure.to_string()))
            };
            $crate::proptest::prop_assert_eq!(read(&store)?, Some(made.clone()), "create then read returns what was created");
            let again = store.create(Created::<$record>::new(first.clone()));
            $crate::proptest::prop_assert_eq!(
                again.map_or_else(|_refused| read(&store), |_twice| Ok(Some(made.clone())))?,
                Some(made.clone()),
                "a refused create leaves the store as it was"
            );
            let revised = made.revised(&second);
            store
                .write(revised.updated())
                .map_err(|failure| $crate::proptest::test_runner::TestCaseError::fail(failure.to_string()))?;
            $crate::proptest::prop_assert_eq!(read(&store)?, Some(revised), "update then read returns the update");
            store
                .write(<$record>::deleted(made.key()))
                .map_err(|failure| $crate::proptest::test_runner::TestCaseError::fail(failure.to_string()))?;
            $crate::proptest::prop_assert_eq!(read(&store)?, None, "delete then read finds nothing");
            store
                .create(Created::<$record>::new(first))
                .map_err(|failure| $crate::proptest::test_runner::TestCaseError::fail(failure.to_string()))?;
            let overwrites = <$record>::overwrites(&argument);
            let at = column.index(overwrites.len());
            let overwritten = overwrites.into_iter().nth(at).map(|overwrite| store.write(overwrite));
            let listed = store.read(<$record>::listed());
            $crate::proptest::prop_assert!(
                listed.is_ok(),
                "the schema refuses what the type refuses: column {at} took {argument:?} ({overwritten:?}) and reads back as {listed:?}"
            );
            }
        );
    };
}
