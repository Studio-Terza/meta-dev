use syntax::{Ident, Macro};

use lint_layout::Name;

use super::Descend;
use super::facts::Facts;

macro_rules! hooks {
    ($($kind:ident $on:ident $off:ident $visit:ident $node:ident;)*) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub enum Kind {
            Macro,
            Ident,
            $($kind,)*
        }

        impl Kind {
            pub const ALL: &'static [Kind] = &[Kind::Macro, Kind::Ident, $(Kind::$kind,)*];
        }

        #[derive(Debug)]
        pub struct ByKind<Value> {
            on_macro: Value,
            on_ident: Value,
            $($on: Value,)*
        }

        impl<Value> ByKind<Value> {
            pub fn filled(mut fill: impl FnMut(Kind) -> Value) -> ByKind<Value> {
                ByKind { on_macro: fill(Kind::Macro), on_ident: fill(Kind::Ident), $($on: fill(Kind::$kind),)* }
            }

            pub fn get(&self, kind: Kind) -> &Value {
                match kind {
                    Kind::Macro => &self.on_macro,
                    Kind::Ident => &self.on_ident,
                    $(Kind::$kind => &self.$on,)*
                }
            }
        }

        pub trait Watch<'ast> {
            fn kinds(&self) -> &[Kind];

            fn on_macro(&mut self, _: &'ast Macro, _: &Facts<'_, Macro>) -> Descend {
                Descend::Into
            }

            fn off_macro(&mut self, _: &'ast Macro, _: &Facts<'_, Macro>) {}

            fn on_ident(&mut self, _: &'ast Ident, _: &Name) -> Descend {
                Descend::Into
            }

            fn off_ident(&mut self, _: &'ast Ident) {}

            $(
                fn $on(&mut self, _: &'ast syntax::$node, _: &Facts<'_, syntax::$node>) -> Descend {
                    Descend::Into
                }

                fn $off(&mut self, _: &'ast syntax::$node, _: &Facts<'_, syntax::$node>) {}
            )*
        }

        impl<'ast, Inner: Watch<'ast> + ?Sized> Watch<'ast> for &mut Inner {
            fn kinds(&self) -> &[Kind] {
                (**self).kinds()
            }

            fn on_macro(&mut self, node: &'ast Macro, facts: &Facts<'_, Macro>) -> Descend {
                (**self).on_macro(node, facts)
            }

            fn off_macro(&mut self, node: &'ast Macro, facts: &Facts<'_, Macro>) {
                (**self).off_macro(node, facts);
            }

            fn on_ident(&mut self, node: &'ast Ident, name: &Name) -> Descend {
                (**self).on_ident(node, name)
            }

            fn off_ident(&mut self, node: &'ast Ident) {
                (**self).off_ident(node);
            }

            $(
                fn $on(&mut self, node: &'ast syntax::$node, facts: &Facts<'_, syntax::$node>) -> Descend {
                    (**self).$on(node, facts)
                }

                fn $off(&mut self, node: &'ast syntax::$node, facts: &Facts<'_, syntax::$node>) {
                    (**self).$off(node, facts);
                }
            )*
        }

        #[macro_export]
        macro_rules! scope_nodes {
            () => {
                fn kinds(&self) -> &[$crate::Kind] {
                    &self.kinds
                }

                fn on_macro(&mut self, node: &'ast syntax::Macro, facts: &$crate::Facts<'_, syntax::Macro>) -> $crate::Descend {
                    self.entered($crate::Kind::Macro, $crate::Address::from(node), &mut |watcher| watcher.on_macro(node, facts))
                }

                fn off_macro(&mut self, node: &'ast syntax::Macro, facts: &$crate::Facts<'_, syntax::Macro>) {
                    self.left($crate::Kind::Macro, $crate::Address::from(node), &mut |watcher| watcher.off_macro(node, facts));
                }

                fn on_ident(&mut self, node: &'ast syntax::Ident, name: &lint_layout::Name) -> $crate::Descend {
                    self.entered($crate::Kind::Ident, $crate::Address::from(node), &mut |watcher| watcher.on_ident(node, name))
                }

                fn off_ident(&mut self, node: &'ast syntax::Ident) {
                    self.left($crate::Kind::Ident, $crate::Address::from(node), &mut |watcher| watcher.off_ident(node));
                }

                $(
                    fn $on(&mut self, node: &'ast syntax::$node, facts: &$crate::Facts<'_, syntax::$node>) -> $crate::Descend {
                        self.entered($crate::Kind::$kind, $crate::Address::from(node), &mut |watcher| watcher.$on(node, facts))
                    }

                    fn $off(&mut self, node: &'ast syntax::$node, facts: &$crate::Facts<'_, syntax::$node>) {
                        self.left($crate::Kind::$kind, $crate::Address::from(node), &mut |watcher| watcher.$off(node, facts));
                    }
                )*
            };
        }

        #[macro_export]
        macro_rules! sweep_nodes {
            () => {
                fn around(&mut self, node: Node<'_>, descend: &mut dyn FnMut(&mut Self)) {
                    match node {
                        Node::Macro(found) => self.macro_around(found, descend),
                        Node::Ident(found) => self.ident_around(found),
                        $(
                            Node::$kind(found) => {
                                let facts = $crate::Facts::new(found);
                                self.around_kind(
                                    $crate::Kind::$kind,
                                    |watcher| watcher.$on(found, &facts),
                                    descend,
                                    |watcher| watcher.$off(found, &facts),
                                );
                            }
                        )*
                    }
                }
            };
        }
    };
}

syntax::node_kinds!(hooks);
