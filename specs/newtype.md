# newtype

Problem: newtypes repeat the same constructors and arithmetic by hand. That hides missing abstractions.

L1 One trait with an associated type expresses every plain newtype; a derive implements it.
L2 Shared behaviour comes from blanket traits, never hand-written on each type.
L3 `repeated_methods` stays a failing gate and does not flag derived code.
D1 `Newtype` trait with `type Inner`, `new`, `into_inner`, and a separate fallible form for validated newtypes, plus `#[derive(Newtype)]`. A derived `Arbitrary` works with it.
D2 Blanket traits for counters, for example increment and count. after: D1
D3 Phantom-tagged generic form, for example `Count<Rounds>`, if it proves simpler than many derived types. Report if not. after: D2
D4 Report-only lint: public `is_*` and `has_*` methods returning `bool` are listed, with a message naming `Option`, an enum, or a proof type such as `NonEmpty`. Blocks nothing.
Out: changing chess, typestate, RAII lints.
