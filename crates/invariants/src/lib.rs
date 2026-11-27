use std::fmt;

pub trait Ignore: Copy + fmt::Display {}

pub trait Ignoring<Value, Failure>: Into<Result<Value, Failure>> {
    fn ignoring(self, _: impl Ignore) -> Option<Value> {
        self.into().into_iter().next()
    }
}

impl<Value, Failure> Ignoring<Value, Failure> for Result<Value, Failure> {}

#[macro_export]
macro_rules! invariants {
    ($(#[$meta:meta])* $vis:vis enum $name:ident { $($variant:ident => $said:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        $vis enum $name {
            $($variant),+
        }

        impl ::core::fmt::Display for $name {
            fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self {
                    $(Self::$variant => formatter.write_str($said)),+
                }
            }
        }
    };
}

#[macro_export]
macro_rules! ignored {
    ($(#[$meta:meta])* $vis:vis enum $name:ident { $($variant:ident => $said:literal),+ $(,)? }) => {
        $crate::invariants! { $(#[$meta])* $vis enum $name { $($variant => $said),+ } }

        impl $crate::Ignore for $name {}
    };
}

#[macro_export]
macro_rules! invariant {
    (broken $named:expr, $detail:expr) => {
        ::core::panic!("broken invariant: {}: {}", $named, $detail)
    };
    (broken $named:expr) => {
        ::core::panic!("broken invariant: {}", $named)
    };
    (holding $result:expr, $named:expr, detailed $(,)?) => {
        match $result {
            Ok(value) => value,
            Err(failure) => $crate::invariant!(broken $named, failure),
        }
    };
    (holding $result:expr, $named:expr $(,)?) => {
        match $result {
            Ok(value) => value,
            Err(_) => $crate::invariant!(broken $named),
        }
    };
    ($holds:expr, $named:expr $(,)?) => {
        match $holds {
            true => {}
            false => $crate::invariant!(broken $named),
        }
    };
}

invariants! {
    enum Invariant {
        LockNotPoisoned =>
            "no thread panics while it holds a lock, so no lock is poisoned; were one, its state may be half-written",
    }
}

pub trait Unpoisoned<Guard>: Into<Result<Guard, std::sync::PoisonError<Guard>>> {
    fn unpoisoned(self) -> Guard {
        invariant!(holding self.into(), Invariant::LockNotPoisoned)
    }
}

impl<Guard> Unpoisoned<Guard> for std::sync::LockResult<Guard> {}
