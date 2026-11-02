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

## History

D3, measured, not built: the phantom form is not simpler. A prototype on main at a4e6e6e moved all 19 `usize` newtypes to `newtype::Count<Tag>`, with hand-written generic impls of `Clone`, `Copy`, `Debug`, `Default`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, `Display`, `Serialize` and `Newtype`. It compiled and passed clippy.

| | derived structs | `Count<Tag>` |
|---|---|---|
| Type items | 19 structs | 1 generic struct, 19 tag enums, 19 aliases, 6 extension traits: 45 |
| Lines declaring the 19 types | 149 | 207, plus 95 for `Count<Tag>`, 4 for a shared trait and 6 of module, re-export and manifest lines |
| Call-site files changed | 0 | 17, each gaining a trait or `Counter` import: +18 −5 lines |
| Whole change | | 42 files, +273 −97, net +176 |
| New blocking lint findings | 0 | 14: 9 `primitive_boundary`, 3 `crate_graph` splits, 1 missing law on the hand-written `Ord`, 1 `usize` field in `Count` itself |

Why: an alias of a foreign generic type cannot have inherent items, so the 7 types with their own methods or constants (`Least`, `Most`, `Depth`, `Rows`, `Stride`, `ItemCount`, mutation-report's `Count`) each needed an extension trait, imported wherever it is called. `Least::allows` and `Most::allows` stopped being `const fn`. Every counter gets every trait, so per-type choices such as no `Ord` or no `Default` are lost. `newtype` would depend on `serde`. Passing the lints would need more code still.
