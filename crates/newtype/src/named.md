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
    #[value(glyph = "*")]
    #[value(rank = 2)]
    On,
    #[name("off")]
    #[value(glyph = "-")]
    #[value(rank = 1)]
    Off,
}

const GLYPH: &str = Lamp::On.glyph();
const RANK: u8 = Lamp::Off.rank();
```

The attribute that supplies a value is `#[value(glyph = "*")]` because a derive can only register helper
attributes by name ahead of time, so a variant cannot be annotated with the accessor's own name.

An accessor value missing on a variant:

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(glyph: &'static str)]
enum Lamp {
    #[name("on")]
    #[value(glyph = "*")]
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
    #[value(glyph = "*")]
    #[value(glyph = "+")]
    On,
    #[name("off")]
    #[value(glyph = "-")]
    Off,
}
```

An accessor value of the wrong literal type:

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(rank: u8)]
enum Lamp {
    #[name("on")]
    #[value(rank = b'a')]
    On,
    #[name("off")]
    #[value(rank = 1)]
    Off,
}
```

An accessor value that no accessor declares:

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
enum Lamp {
    #[name("on")]
    #[value(glyph = "*")]
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
    #[value(rank = 2)]
    On,
}
```

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(name: &'static str)]
enum Lamp {
    #[name("on")]
    #[value(name = "x")]
    On,
}
```

An accessor of a type that is not a literal type:

```compile_fail
#[derive(newtype::Named, strum::VariantArray, Clone, Copy)]
#[accessor(rank: core::primitive::u8)]
enum Lamp {
    #[name("on")]
    #[value(rank = 2)]
    On,
    #[name("off")]
    #[value(rank = 1)]
    Off,
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
