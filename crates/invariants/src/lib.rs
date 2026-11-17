#[macro_export]
macro_rules! invariant {
    (broken $named:expr) => {
        ::core::panic!("broken invariant: {}", $named)
    };
    ($holds:expr, $named:expr $(,)?) => {
        match $holds {
            true => {}
            false => $crate::invariant!(broken $named),
        }
    };
}
