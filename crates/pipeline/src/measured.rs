#[macro_export]
macro_rules! measured {
    ($(#[$meta:meta])* $vis:vis struct $name:ident { $($field_vis:vis $field:ident: $kind:ty),* $(,)? }) => {
        $(#[$meta])*
        $vis struct $name {
            $($field_vis $field: $kind,)*
        }

        impl ::core::default::Default for $name {
            fn default() -> $name {
                $name { $($field: ::core::default::Default::default(),)* }
            }
        }

        impl $crate::Measure for $name {
            fn joined(self, other: $name) -> $name {
                $name { $($field: $crate::Measure::joined(self.$field, other.$field),)* }
            }
        }
    };
    ($(#[$meta:meta])* $vis:vis struct $name:ident($inner_vis:vis $inner:ty);) => {
        $(#[$meta])*
        $vis struct $name($inner_vis $inner);

        impl ::core::default::Default for $name {
            fn default() -> $name {
                $name(::core::default::Default::default())
            }
        }

        impl $crate::Measure for $name {
            fn joined(self, other: $name) -> $name {
                $name($crate::Measure::joined(self.0, other.0))
            }
        }
    };
}
