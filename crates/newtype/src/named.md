Compile checks for `#[derive(Named)]`. The first block compiles; each later block differs from it by one
defect and must not compile.

Every unit variant names itself once:

```
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
enum Lamp {
    #[name("on")]
    On,
    #[name("off")]
    Off,
}

const LIT: &str = Lamp::On.name();
```

Other per-variant values become const accessors. The enum declares each accessor, any literal-typed const
(`&'static str`, an integer type, `bool` or `char`), and every variant supplies a value:

```
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(glyph: &'static str)]
#[accessor(rank: u8)]
enum Lamp {
    #[name("on")]
    #[accessor_value(glyph = "*")]
    #[accessor_value(rank = 2)]
    On,
    #[name("off")]
    #[accessor_value(glyph = "-")]
    #[accessor_value(rank = 1)]
    Off,
}

const GLYPH: &str = Lamp::On.glyph();
const RANK: u8 = Lamp::Off.rank();
```

The attribute that supplies a value is `#[accessor_value(glyph = "*")]` because a derive can only register helper
attributes by name ahead of time, so a variant cannot be annotated with the accessor's own name. It is not called
`value` because `clap::ValueEnum` registers a helper attribute named `value`, and two derives on one enum cannot
both register it: with `accessor_value`, an enum can derive `Named` and `clap::ValueEnum` together. This crate has
no clap dependency, so the reason is documented here and not compiled.

A `u8` accessor takes an integer literal or a byte literal such as `b'a'`; a `char` accessor takes a char literal.
An `&'static str` accessor may also be spelled `&'static core::primitive::str`:

```
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(byte: u8)]
#[accessor(text: &'static core::primitive::str)]
#[accessor(path: &'static ::std::primitive::str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(byte = b'a')]
    #[accessor_value(text = "*")]
    #[accessor_value(path = "+")]
    On,
    #[name("off")]
    #[accessor_value(byte = 1)]
    #[accessor_value(text = "-")]
    #[accessor_value(path = "-")]
    Off,
}

const BYTE: u8 = Lamp::On.byte();
```

An accessor value missing on a variant:

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(glyph: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(glyph = "*")]
    On,
    #[name("off")]
    Off,
}
```

An accessor value given twice:

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(glyph: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(glyph = "*")]
    #[accessor_value(glyph = "+")]
    On,
    #[name("off")]
    #[accessor_value(glyph = "-")]
    Off,
}
```

An accessor value of the wrong literal type, here a char literal for a `u8`, which is refused with "this literal
does not have the accessor's declared type":

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(rank: u8)]
enum Lamp {
    #[name("on")]
    #[accessor_value(rank = 'a')]
    On,
    #[name("off")]
    #[accessor_value(rank = 1)]
    Off,
}
```

An accessor value that no accessor declares:

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
enum Lamp {
    #[name("on")]
    #[accessor_value(glyph = "*")]
    On,
    #[name("off")]
    Off,
}
```

Two accessors with one name, or an accessor called `name`:

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(rank: u8)]
#[accessor(rank: u16)]
enum Lamp {
    #[name("on")]
    #[accessor_value(rank = 2)]
    On,
}
```

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(name: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(name = "x")]
    On,
}
```

An accessor takes no name that a method of `Clone`, `PartialEq`, `PartialOrd`, `Ord`, `Hash`, `Debug`, `Display`,
`ToString`, `FromStr`, `From`, `Into`, `TryFrom`, `TryInto`, `Borrow`, `BorrowMut`, `AsRef`, `ToOwned`, `Any`, `Default` or `VariantArray` has, nor `name`, because the inherent const method would shadow that trait method on the enum. Each name
is refused:

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(clone: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(clone = "x")]
    On,
}
```

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(clone_from: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(clone_from = "x")]
    On,
}
```

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(eq: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(eq = "x")]
    On,
}
```

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(ne: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(ne = "x")]
    On,
}
```

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(partial_cmp: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(partial_cmp = "x")]
    On,
}
```

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(lt: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(lt = "x")]
    On,
}
```

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(le: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(le = "x")]
    On,
}
```

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(gt: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(gt = "x")]
    On,
}
```

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(ge: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(ge = "x")]
    On,
}
```

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(cmp: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(cmp = "x")]
    On,
}
```

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(max: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(max = "x")]
    On,
}
```

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(min: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(min = "x")]
    On,
}
```

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(clamp: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(clamp = "x")]
    On,
}
```

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(hash: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(hash = "x")]
    On,
}
```

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(hash_slice: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(hash_slice = "x")]
    On,
}
```

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(fmt: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(fmt = "x")]
    On,
}
```

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(to_string: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(to_string = "x")]
    On,
}
```

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(from_str: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(from_str = "x")]
    On,
}
```

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(into: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(into = "x")]
    On,
}
```

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(try_into: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(try_into = "x")]
    On,
}
```

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(borrow: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(borrow = "x")]
    On,
}
```

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(type_id: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(type_id = "x")]
    On,
}
```

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(from: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(from = "x")]
    On,
}
```

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(try_from: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(try_from = "x")]
    On,
}
```

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(VARIANTS: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(VARIANTS = "x")]
    On,
}
```

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(borrow_mut: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(borrow_mut = "x")]
    On,
}
```

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(as_ref: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(as_ref = "x")]
    On,
}
```

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(to_owned: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(to_owned = "x")]
    On,
}
```

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(clone_into: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(clone_into = "x")]
    On,
}
```

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(default: &'static str)]
enum Lamp {
    #[name("on")]
    #[accessor_value(default = "x")]
    On,
}
```

An accessor of a primitive type spelled with its path, or with a raw name, compiles:

```
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(r#rank: core::primitive::u8)]
enum Lamp {
    #[name("on")]
    #[accessor_value(rank = 2)]
    On,
    #[name("off")]
    #[accessor_value(r#rank = 1)]
    Off,
}
```

An accessor of a type that is not a literal type:

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(rank: String)]
enum Lamp {
    #[name("on")]
    #[accessor_value(rank = 2)]
    On,
    #[name("off")]
    #[accessor_value(rank = 1)]
    Off,
}
```

An accessor on a variant, where it would be ignored:

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
enum Lamp {
    #[name("on")]
    #[accessor(glyph: &'static str)]
    On,
    #[name("off")]
    Off,
}
```

A value on the enum, where it would be ignored:

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor_value(glyph = "*")]
enum Lamp {
    #[name("on")]
    On,
    #[name("off")]
    Off,
}
```

A variant list that repeats a variant after listing them all:

```compile_fail
#[derive(newtype::Named, Clone, Copy)]
enum Lamp {
    #[name("on")]
    On,
    #[name("off")]
    Off,
}

impl strum::VariantArray for Lamp {
    const VARIANTS: &'static [Self] = &[Lamp::On, Lamp::Off, Lamp::On];
}
```

A variant list that repeats a variant and so leaves another out:

```compile_fail
#[derive(newtype::Named, Clone, Copy)]
enum Lamp {
    #[name("on")]
    On,
    #[name("off")]
    Off,
}

impl strum::VariantArray for Lamp {
    const VARIANTS: &'static [Self] = &[Lamp::On, Lamp::On];
}
```

A variant list that leaves a variant out, so `FromStr` could not read its name back:

```compile_fail
#[derive(newtype::Named, Clone, Copy)]
enum Lamp {
    #[name("on")]
    On,
    #[name("off")]
    Off,
}

impl strum::VariantArray for Lamp {
    const VARIANTS: &'static [Self] = &[Lamp::On];
}
```

An enum that is not Copy. Named reports "Named needs an enum that is Copy: add Copy to its derive list". The
check is a trait bound on the enum, so Copy from a derive, from a hand-written `impl Copy` or from another
macro all satisfy it:

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone)]
enum Lamp {
    #[name("on")]
    On,
    #[name("off")]
    Off,
}
```

A variant without a name:

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
enum Lamp {
    #[name("on")]
    On,
    Off,
}
```

Two variants with one name:

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
enum Lamp {
    #[name("on")]
    On,
    #[name("on")]
    Off,
}
```

A name on the enum itself:

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[name("lamp")]
enum Lamp {
    #[name("on")]
    On,
    #[name("off")]
    Off,
}
```

A variant with a field:

```compile_fail
#[derive(newtype::Named, Clone, Copy)]
enum Lamp {
    #[name("on")]
    On(u8),
    #[name("off")]
    Off,
}

impl strum::VariantArray for Lamp {
    const VARIANTS: &'static [Self] = &[Lamp::Off];
}
```

A variant named twice:

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
enum Lamp {
    #[name("on")]
    #[name("lit")]
    On,
    #[name("off")]
    Off,
}
```

A struct:

```compile_fail
#[derive(newtype::Named, Clone, Copy)]
struct Lamp;
```

Without the variant list from strum:

```compile_fail
#[derive(newtype::Named, Clone, Copy)]
enum Lamp {
    #[name("on")]
    On,
    #[name("off")]
    Off,
}
```

An enum with no variants:

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
enum Lamp {}
```
