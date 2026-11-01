#![doc = include_str!("compile_fail.md")]

#[macro_export]
macro_rules! states {
    (
        $(#[$meta:meta])*
        $vis:vis mod $lifecycle:ident {
            $($kind:ident $state:ident { $($edge:ident -> $target:ident),* $(,)? })*
        }
    ) => {
        $(#[$meta])*
        $vis mod $lifecycle {
            type Brand<'id> = ::core::marker::PhantomData<fn(&'id ()) -> &'id ()>;

            #[must_use = "return the witness: it proves the lifecycle reached a terminal state"]
            #[derive(Debug)]
            pub struct Finished<'id>(Brand<'id>);

            $(
                #[must_use = "a lifecycle state must be taken to a terminal state"]
                #[derive(Debug)]
                pub struct $state<'id>(Brand<'id>);

                impl<'id> $state<'id> {
                    $(
                        pub fn $edge(self) -> $target<'id> {
                            $target(self.0)
                        }
                    )*
                }

                $crate::states!(@ $kind $state);
            )*
        }
    };
    (@ start $state:ident) => {
        impl $state<'_> {
            pub fn scope(body: impl for<'id> FnOnce($state<'id>) -> Finished<'id>) {
                let Finished(_) = body($state(::core::marker::PhantomData));
            }
        }
    };
    (@ state $state:ident) => {};
    (@ terminal $state:ident) => {
        impl<'id> $state<'id> {
            pub fn finish(self) -> Finished<'id> {
                Finished(self.0)
            }
        }
    };
}
