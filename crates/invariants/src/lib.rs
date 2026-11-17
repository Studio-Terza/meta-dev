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
macro_rules! invariant {
    (broken $named:expr, $detail:expr) => {
        ::core::panic!("broken invariant: {}: {}", $named, $detail)
    };
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
