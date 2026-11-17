use std::fmt;

pub trait Ignore: Copy + fmt::Display {}

pub trait Ignoring<Value, Failure>: Into<Result<Value, Failure>> {
    fn ignoring(self, _: impl Ignore) -> Option<Value> {
        match self.into() {
            Ok(value) => Some(value),
            Err(failure) => Self::dropped(failure),
        }
    }

    fn dropped(failure: Failure) -> Option<Value> {
        drop(failure);
        None
    }
}

impl<Value, Failure> Ignoring<Value, Failure> for Result<Value, Failure> {}

#[macro_export]
macro_rules! invariant {
    (broken $named:expr) => {
        ::core::panic!("broken invariant: {}", $named)
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
