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
    (
        $(#[$meta:meta])*
        $vis:vis stored $lifecycle:ident in $proofs:ident {
            $($kind:ident $state:ident $(as $targets:ident)? { $($edge:ident -> $target:ident),* $(,)? })*
        }
    ) => {
        $(#[$meta])*
        #[derive(::named_derive::Named, Clone, Copy, Debug, PartialEq, Eq)]
        $vis enum $lifecycle {
            $($state),*
        }

        impl ::store_ports::Lifecycle for $lifecycle {
            const EDGES: &'static [::store_ports::Edge<$lifecycle>] = &[
                $($(::store_ports::Edge::new($lifecycle::$state, $lifecycle::$target),)*)*
            ];
        }

        $vis mod $proofs {
            $(
                $crate::states!(@ held $kind $lifecycle $state { $($edge -> $target),* });
                $crate::states!(@ choosing $lifecycle $state [$($targets)?] { $($target),* });
            )*
        }
    };
    (@ choosing $lifecycle:ident $state:ident [] { $($target:ident),* }) => {};
    (@ choosing $lifecycle:ident $state:ident [$targets:ident] { $($target:ident),* }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum $targets {
            $($target),*
        }

        impl ::core::convert::From<$targets> for super::$lifecycle {
            fn from(target: $targets) -> super::$lifecycle {
                match target {
                    $($targets::$target => super::$lifecycle::$target),*
                }
            }
        }

        impl $state {
            pub fn moved(self, to: $targets) -> ::store_ports::Moved<super::$lifecycle> {
                ::store_ports::Moved::new(self.0, ::store_ports::Edge::new(super::$lifecycle::$state, super::$lifecycle::from(to)))
            }
        }
    };
    (@ held terminal $lifecycle:ident $state:ident { $($edge:ident -> $target:ident),* }) => {};
    (@ held $kind:ident $lifecycle:ident $state:ident { $($edge:ident -> $target:ident),* }) => {
        #[must_use = "a state read from the store is held to move it"]
        #[derive(Debug, PartialEq, Eq)]
        pub struct $state(<super::$lifecycle as ::store_ports::Staged>::Key);

        impl $state {
            pub(crate) fn held(key: <super::$lifecycle as ::store_ports::Staged>::Key) -> $state {
                $state(key)
            }

            #[must_use]
            pub fn key(&self) -> &<super::$lifecycle as ::store_ports::Staged>::Key {
                &self.0
            }

            $(
                pub fn $edge(self) -> ::store_ports::Moved<super::$lifecycle> {
                    ::store_ports::Moved::new(self.0, ::store_ports::Edge::new(super::$lifecycle::$state, super::$lifecycle::$target))
                }
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
