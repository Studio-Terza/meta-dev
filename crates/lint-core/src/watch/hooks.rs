use syntax::{Ident, Macro, Node};

use algebra::Monoid;
use lint_layout::Name;

use super::Descend;
use super::facts::Facts;
use crate::contribution::{Around, Contributed, Contributing, Contribution, Scope, Visit, Walked};
use crate::kind_index::Indexed;
use crate::macros::Macros;
use crate::source_file::SourceFile;

macro_rules! kind_bits {
    (@arms $value:expr; [$($done:tt)*]; $shift:expr;) => {
        match $value {
            $($done)*
        }
    };
    (@arms $value:expr; [$($done:tt)*]; $shift:expr; $kind:ident $($rest:ident)*) => {
        kind_bits!(@arms $value; [$($done)* Kind::$kind => 1_u128 << ($shift),]; $shift + 1_u32; $($rest)*)
    };
}

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

            pub(crate) fn bit(self) -> u128 {
                kind_bits!(@arms self; []; 0_u32; Macro Ident $($kind)*)
            }

            pub(crate) fn of(node: Node<'_>) -> Kind {
                match node {
                    Node::Macro(_) => Kind::Macro,
                    Node::Ident(_) => Kind::Ident,
                    $(Node::$kind(_) => Kind::$kind,)*
                }
            }
        }

        pub trait Contribute {
            type Open: Monoid;
            type Part: Monoid;

            fn kinds(&self) -> &[Kind];

            fn scopes(&self) -> &[Scope] {
                &[]
            }

            fn on_macro(&self, _: &Macro, _: &Facts<'_, Macro>, _: &Around<'_, Self::Open>) -> Contributed<Self::Open, Self::Part> {
                Contributed::descending(Descend::Into)
            }

            fn off_macro(&self, _: &Macro, _: &Facts<'_, Macro>, inner: Contribution<Self::Open, Self::Part>) -> Self::Part {
                inner.concluded(|_, part| part)
            }

            fn on_ident(&self, _: &Ident, _: &Name, _: &Around<'_, Self::Open>) -> Contributed<Self::Open, Self::Part> {
                Contributed::descending(Descend::Into)
            }

            fn off_ident(&self, _: &Ident, inner: Contribution<Self::Open, Self::Part>) -> Self::Part {
                inner.concluded(|_, part| part)
            }

            $(
                fn $on(&self, _: &syntax::$node, _: &Facts<'_, syntax::$node>, _: &Around<'_, Self::Open>) -> Contributed<Self::Open, Self::Part> {
                    Contributed::descending(Descend::Into)
                }

                fn $off(
                    &self,
                    _: &syntax::$node,
                    _: &Facts<'_, syntax::$node>,
                    inner: Contribution<Self::Open, Self::Part>,
                ) -> Self::Part {
                    inner.concluded(|_, part| part)
                }
            )*
        }

        impl<Contributor: Contribute> Contributing<Contributor> {
            pub(crate) fn dispatched(
                &self,
                file: Option<&SourceFile>,
                node: Node<'_>,
                indexed: Option<Indexed<'_>>,
                around: &Around<'_, Contributor::Open>,
            ) -> Walked<Contributor::Open, Contributor::Part> {
                match node {
                    Node::Macro(found) => self.macro_walked(file, found, indexed, around),
                    Node::Ident(found) => self.ident_walked(file, found, indexed, around),
                    $(Node::$kind(found) => match self.watches(Kind::$kind) {
                        None => self.unwatched(file, node, indexed.map(Indexed::children), around),
                        Some(kind) => self.$visit(file, kind, found, indexed, around),
                    },)*
                }
            }

            $(
                fn $visit(
                    &self,
                    file: Option<&SourceFile>,
                    kind: Kind,
                    found: &syntax::$node,
                    indexed: Option<Indexed<'_>>,
                    around: &Around<'_, Contributor::Open>,
                ) -> Walked<Contributor::Open, Contributor::Part> {
                    let facts = Facts::new(found);
                    self.entered(
                        Visit::new(file, kind, Node::$kind(found), indexed),
                        around,
                        |contributor: &Contributor, around: &Around<'_, Contributor::Open>| contributor.$on(found, &facts, around),
                        |contributor: &Contributor, inner| contributor.$off(found, &facts, inner),
                    )
                }
            )*

            fn macro_walked(
                &self,
                file: Option<&SourceFile>,
                found: &Macro,
                indexed: Option<Indexed<'_>>,
                around: &Around<'_, Contributor::Open>,
            ) -> Walked<Contributor::Open, Contributor::Part> {
                file.map_or_default(|source| Macros::pin(source, found));
                let node = Node::Macro(found);
                match self.watches(Kind::Macro) {
                    None => self.unwatched(file, node, indexed.map(Indexed::children), around),
                    Some(kind) => self.macro_entered(Visit::new(file, kind, node, indexed), found, around),
                }
            }

            fn macro_entered(
                &self,
                visit: Visit<'_, '_, '_>,
                found: &Macro,
                around: &Around<'_, Contributor::Open>,
            ) -> Walked<Contributor::Open, Contributor::Part> {
                let facts = Facts::new(found);
                self.entered(
                    visit,
                    around,
                    |contributor: &Contributor, around: &Around<'_, Contributor::Open>| contributor.on_macro(found, &facts, around),
                    |contributor: &Contributor, inner| contributor.off_macro(found, &facts, inner),
                )
            }

            fn ident_walked(
                &self,
                file: Option<&SourceFile>,
                found: &Ident,
                indexed: Option<Indexed<'_>>,
                around: &Around<'_, Contributor::Open>,
            ) -> Walked<Contributor::Open, Contributor::Part> {
                let name = self.watches(Kind::Ident).map(|_| Name::from(found));
                name.map_or_default(|named| {
                    self.entered(
                        Visit::new(file, Kind::Ident, Node::Ident(found), indexed),
                        around,
                        |contributor: &Contributor, around: &Around<'_, Contributor::Open>| contributor.on_ident(found, &named, around),
                        |contributor: &Contributor, inner| contributor.off_ident(found, inner),
                    )
                })
            }
        }
    };
}

syntax::node_kinds!(hooks);
