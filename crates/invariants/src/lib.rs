use std::fmt;

pub trait Ignore: Copy + fmt::Display {}

pub trait Ignoring<Value>: IntoIterator<Item = Value> + Sized {
    fn ignoring(self, _: impl Ignore) -> Option<Value> {
        self.into_iter().next()
    }
}

impl<Value, Failure> Ignoring<Value> for Result<Value, Failure> {}

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
