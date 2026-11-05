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
