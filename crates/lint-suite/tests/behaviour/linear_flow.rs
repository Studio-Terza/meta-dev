use lint_core::Lint;
use lint_linear_flow::{FlowShapes, IndependentSelections, LinearBodies, PlainConditions, PureRouters};

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

const PIPELINE: &str =
    "fn f(items: &[u8]) -> Vec<u8> {\n    let doubled = items.iter().map(|item| item * 2);\n    doubled.collect()\n}\n";
const IF: &str = "fn f(on: Switch) -> u8 {\n    if on.lit() { 1 } else { 0 }\n}\n";
const MATCH_AFTER_LET: &str = "fn f(on: Switch) -> u8 {\n    let lit = on.lit();\n    match lit {\n        Lit::On => 1,\n        Lit::Off => 0,\n    }\n}\n";
const ROUTER: &str =
    "fn f(on: Switch) -> u8 {\n    match on {\n        Switch::On => g(),\n        Switch::Off => 0,\n    }\n}\n";
const IF_LET_ROUTER: &str = "fn f(found: Option<u8>) -> u8 {\n    if let Some(value) = found { value } else { 0 }\n}\n";
const IF_LET_INSIDE: &str =
    "fn f(found: Option<u8>) -> u8 {\n    let seen = found;\n    if let Some(value) = seen { value } else { 0 }\n}\n";
const IF_IN_ARM: &str = "fn f(on: Switch) -> u8 {\n    match on {\n        Switch::On => g(if p { 1 } else { 2 }),\n        Switch::Off => 0,\n    }\n}\n";
const LOOPS: &str =
    "fn f() {\n    for _ in [1] {}\n    while go() {}\n    while let Some(_) = next() {}\n    loop {}\n}\n";
const LET_ELSE: &str =
    "fn f(found: Option<u8>) -> u8 {\n    let Some(value) = found else { panic!() };\n    value\n}\n";
const CLOSURE: &str = "fn f(items: &[Switch]) -> Vec<u8> {\n    items.iter().map(|on| match on {\n        Switch::On => 1,\n        Switch::Off => 0,\n    }).collect()\n}\n";
const CONST_FN: &str = "const fn f(on: bool) -> u8 {\n    if on { 1 } else { 0 }\n}\n";
const NESTED: &str = "fn f() -> u8 {\n    fn g(on: Switch) -> u8 {\n        match on {\n            Switch::On => 1,\n            Switch::Off => 0,\n        }\n    }\n    let routed = g(Switch::On);\n    if routed.lit() { 1 } else { 0 }\n}\n";
const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    fn f() {\n        for _ in [1] {}\n    }\n}\n";
const INTEGRATION: &str = "#[test]\nfn f() {\n    loop {}\n}\n";
const IN_MACRO: &str = "fn f() -> Vec<u8> {\n    vec![if p { 1 } else { 2 }]\n}\n";
const METHODS: &str = "trait T {\n    fn f(&self) {\n        loop {}\n    }\n}\nimpl T for A {\n    fn f(&self) {\n        loop {}\n    }\n}\n";

const IN_MACRO_RULES: &str =
    "macro_rules! m {\n    () => {\n        fn f() {\n            loop {}\n        }\n    };\n}\n";
const WRAPPED: &str = "fn f(count: &u8) -> u8 {\n    match count {\n        &0 => g(),\n        n @ 1 => h(n),\n        (2) => k(),\n        3 | 4 => m(),\n        _ => 0,\n    }\n}\n";

const GUARD: &str =
    "fn f(on: Switch) -> u8 {\n    match on {\n        Switch::On if p => 1,\n        _ => 0,\n    }\n}\n";
const LITERAL: &str = "fn f(count: u8) -> u8 {\n    match count {\n        0 => g(),\n        1..=3 => h(),\n        _ => k(),\n    }\n}\n";
const BOOLEAN: &str = "fn f(on: bool) -> u8 {\n    match on {\n        true => 1,\n        false => 0,\n    }\n}\n";
const TUPLE: &str = "fn f(a: Switch, b: Switch) -> u8 {\n    match (a, b) {\n        (Switch::On, _) => 1,\n        _ => 0,\n    }\n}\n";
const NESTED_PATTERNS: &str = "fn f(found: Option<Pair>) -> u8 {\n    match found {\n        Some(Pair { left: 0, .. }) => 1,\n        Some(Pair { left: (a, b), .. }) => g(a, b),\n        Some(ref inner @ Pair::Wide(_)) | Some(inner) => h(inner),\n        None => 0,\n    }\n}\n";
const ARMS: &str = "fn f(on: Switch) -> u8 {\n    match on {\n        Switch::On => {\n            let x = g();\n            h(x)\n        }\n        Switch::Off => a + b,\n        Switch::Dim => { g() }\n        Switch::Half => { g(); }\n        Switch::Lit => {}\n        Switch::Low => (k()?),\n        Switch::Out => { println!(\"x\") }\n    }\n}\n";
const ELSE_IF: &str =
    "fn f(found: Option<u8>) -> u8 {\n    if let Some(value) = found { value } else if p { 1 } else { 0 }\n}\n";
const IF_LET_LITERAL: &str =
    "fn f(found: Option<u8>) -> u8 {\n    if let Some(0) = found {\n        let x = g();\n        x\n    }\n}\n";
const GUARD_OUTSIDE: &str = "fn f(on: Switch) -> u8 {\n    let lit = on;\n    match lit {\n        Switch::On if p => 1,\n        _ => 0,\n    }\n}\n";

const OPERATORS: &str = "fn f(items: &[Item]) -> usize {\n    let kept = items.iter().filter(|item| item.on() && item.lit());\n    let other = kept.filter(|item| item.on() || !item.lit());\n    other.filter(|item| item.size() == -1 && *item.next() != 2).count()\n}\n";
const OPERATOR_IN_TEST: &str = "#[cfg(test)]\nmod tests {\n    fn f() {\n        assert!(a || b);\n    }\n}\n";

const SHAPES: &str = "struct A;\nimpl A {\n    fn piped(&self) -> u8 {\n        g().filter(|_| p).h()\n    }\n    fn routed(&self, on: Switch) -> u8 {\n        match on {\n            Switch::On => 1,\n            Switch::Off => 0,\n        }\n    }\n    fn tangled(&self) -> u8 {\n        if p { 1 } else { 0 }\n    }\n    const fn fixed() -> u8 {\n        if p { 1 } else { 0 }\n    }\n}\n";

const FROM_BOOL: &str = "impl From<bool> for Lit {\n    fn from(on: bool) -> Lit {\n        if on { Lit::On } else { Lit::Off }\n    }\n}\n";
const FROM_BOOL_INLINE: &str = "mod boundary {\n    impl From<bool> for Lit {\n        fn from(on: bool) -> Lit {\n            if on { Lit::On } else { Lit::Off }\n        }\n    }\n}\n";
const FROM_BOOL_AFTER_LET: &str = "impl From<bool> for Lit {\n    fn from(on: bool) -> Lit {\n        let lit = on;\n        if lit { Lit::On } else { Lit::Off }\n    }\n}\n";
const FROM_BOOL_JOINED: &str = "impl From<bool> for Lit {\n    fn from(on: bool) -> Lit {\n        if on && READY { Lit::On } else { Lit::Off }\n    }\n}\n";
const FROM_BOOL_ELSE_IF: &str = "impl From<bool> for Lit {\n    fn from(on: bool) -> Lit {\n        if on { Lit::On } else if READY { Lit::Dim } else { Lit::Off }\n    }\n}\n";
const FROM_BOOL_NESTED: &str = "impl From<bool> for Lit {\n    fn from(on: bool) -> Lit {\n        if on { Lit::On } else { if READY { Lit::Dim } else { Lit::Off } }\n    }\n}\n";
const FROM_BOOL_HELPER: &str = "impl From<bool> for Lit {\n    fn from(on: bool) -> Lit {\n        Lit::lit(on)\n    }\n}\nimpl Lit {\n    fn lit(on: bool) -> Lit {\n        if on { Lit::On } else { Lit::Off }\n    }\n}\n";
const FROM_BOOL_CONST: &str = "impl From<bool> for Lit {\n    fn from(on: bool) -> Lit {\n        if READY { Lit::On } else { Lit::Off }\n    }\n}\n";
const FROM_BOOL_PICK: &str = "impl From<bool> for Lit {\n    fn from(on: bool) -> Lit {\n        fn pick(x: bool) -> Lit { if x { Lit::On } else { Lit::Off } }\n        pick(on)\n    }\n}\n";
const FROM_BOOL_COMPOUND: &str = "impl From<bool> for Lit {\n    fn from(on: bool) -> Lit {\n        if on { let lit = Lit::On; lit.dimmed() } else { Lit::Off }\n    }\n}\n";
const FROM_BOOL_STRUCT: &str = "pub struct Delay {\n    millis: u64,\n}\nimpl From<bool> for Delay {\n    fn from(on: bool) -> Delay {\n        if on { Delay::from_millis(500) } else { Delay::from_millis(0) }\n    }\n}\n";
const FROM_COUNT: &str = "impl From<u8> for Lit {\n    fn from(count: u8) -> Lit {\n        if count > 0 { Lit::On } else { Lit::Off }\n    }\n}\n";
const FROM_BOOL_MATCHED: &str = "impl From<bool> for Lit {\n    fn from(on: bool) -> Lit {\n        match on {\n            true => Lit::On,\n            false => Lit::Off,\n        }\n    }\n}\n";

const FROM_BOOL_CONST_FN: &str = "impl From<bool> for Lit {\n    const C: u8 = {\n        fn helper(b: bool) -> Lit { if b { Lit::On } else { Lit::Off } }\n        0\n    };\n    fn from(on: bool) -> Lit {\n        if on { Lit::On } else { Lit::Off }\n    }\n}\n";
const FROM_BOOL_SELF: &str = "impl From<bool> for Lit {\n    fn from(on: bool) -> Self {\n        if on { Self::On } else { Lit::Off }\n    }\n}\n";
const FROM_BOOL_MODULE: &str = "impl From<bool> for Level {\n    fn from(on: bool) -> Level {\n        if on { limits::HIGH } else { limits::LOW }\n    }\n}\n";
const FROM_BOOL_LIMIT: &str = "impl From<bool> for Lit {\n    fn from(on: bool) -> Lit {\n        if on { Lit::MAX_LIMIT } else { Lit::MIN_FLOOR }\n    }\n}\n";
const FROM_BOOL_FAMILY: &str = "impl From<bool> for Family {\n    fn from(six: bool) -> Family {\n        if six { Family::V6 } else { Family::V4 }\n    }\n}\n";
const FROM_BOOL_LOWER: &str = "impl From<bool> for Lit {\n    fn from(on: bool) -> Lit {\n        if on { Lit::on } else { Lit::off }\n    }\n}\n";
const FROM_BOOL_SAME: &str =
    "impl From<bool> for Lit {\n    fn from(on: bool) -> Lit {\n        if on { Lit::On } else { Lit::On }\n    }\n}\n";

const FROM_BOOL_SIDE: &str =
    "impl From<bool> for Side {\n    fn from(b: bool) -> Side {\n        if b { Side::A } else { Side::B }\n    }\n}\n";
const FROM_BOOL_FOREIGN: &str = "impl From<bool> for Lit {\n    fn from(on: bool) -> Lit {\n        if on { Other::On } else { Other::Off }\n    }\n}\n";

const FROM_BOOL_PORTS_INLINE: &str = "mod ports {\n    impl From<bool> for Lit {\n        fn from(on: bool) -> Lit {\n            if on { Lit::On } else { Lit::Off }\n        }\n    }\n}\n";
const FROM_BOOL_TWO_INPUTS: &str = "impl From<bool> for Lit {\n    fn from(on: bool, extra: bool) -> Lit {\n        if on { Lit::On } else { Lit::Off }\n    }\n}\n";

const TYPE_DECLS: &str = "pub enum Lit { On, Off, Dim }\npub enum Family { V4, V6 }\npub enum Side { A, B }\n";
const TYPES: Source = Source::new(Located::new("crates/x/src/types.rs"), Text::new(TYPE_DECLS));
const STRUCT_LIT: Source = Source::new(
    Located::new("crates/x/src/struct_lit.rs"),
    Text::new(
        "pub struct Lit(u8);\nimpl Lit {\n    pub const ON: Lit = Lit(1);\n    pub const OFF: Lit = Lit(0);\n}\n",
    ),
);
const TWIN_LIT: Source = Source::new(Located::new("crates/x/src/twin.rs"), Text::new("pub enum Lit { On, Off }\n"));
const STRUCT_TWIN_LIT: Source =
    Source::new(Located::new("crates/x/src/wire.rs"), Text::new("pub struct Lit(pub u8);\n"));
const PARTIAL_TWIN_LIT: Source =
    Source::new(Located::new("crates/x/src/partial.rs"), Text::new("pub enum Lit { On }\n"));
const CFG_TWIN_LIT: Source = Source::new(
    Located::new("crates/x/src/types.rs"),
    Text::new("#[cfg(unix)]\npub enum Lit { On, Off }\n#[cfg(not(unix))]\npub enum Lit { On, Off }\n"),
);
const STRUCT_DELAY: Source = Source::new(
    Located::new("crates/x/src/struct_delay.rs"),
    Text::new(
        "pub struct Delay(u64);\nimpl Delay {\n    pub const SHORT: Delay = Delay(1);\n    pub const LONG: Delay = Delay(9);\n}\n",
    ),
);
const FROM_BOOL_ONOFF: &str = "impl From<bool> for Lit {\n    fn from(on: bool) -> Lit {\n        if on { Lit::ON } else { Lit::OFF }\n    }\n}\n";
const FROM_BOOL_DELAY: &str = "impl From<bool> for Delay {\n    fn from(slow: bool) -> Delay {\n        if slow { Delay::LONG } else { Delay::SHORT }\n    }\n}\n";
const FROM_BOOL_GONE: &str = "impl From<bool> for Lit {\n    fn from(on: bool) -> Lit {\n        if on { Lit::On } else { Lit::Gone }\n    }\n}\n";

const MACRO_LIT: Source = Source::new(
    Located::new("crates/x/src/made.rs"),
    Text::new("macro_rules! make {\n    () => {\n        pub enum Lit { On, Off }\n    };\n}\nmake!();\n"),
);
const FROM_BOOL_IN_TESTS: &str = "#[cfg(test)]\nmod tests {\n    enum Lit { On, Off }\n    impl From<bool> for Lit {\n        fn from(on: bool) -> Lit {\n            if on { Lit::On } else { Lit::Off }\n        }\n    }\n}\n";

const LINEAR: [Case; 51] = [
    Case::new(Title::new("pipeline"), &[Source::new(Located::new("a.rs"), Text::new(PIPELINE))], Flagged::new(&[])),
    Case::new(Title::new("if"), &[Source::new(Located::new("a.rs"), Text::new(IF))], Flagged::new(&["a.rs:2"])),
    Case::new(
        Title::new("match after a let"),
        &[Source::new(Located::new("a.rs"), Text::new(MATCH_AFTER_LET))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(Title::new("router"), &[Source::new(Located::new("a.rs"), Text::new(ROUTER))], Flagged::new(&[])),
    Case::new(
        Title::new("if let router"),
        &[Source::new(Located::new("a.rs"), Text::new(IF_LET_ROUTER))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("if let after a let"),
        &[Source::new(Located::new("a.rs"), Text::new(IF_LET_INSIDE))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("if inside an arm"),
        &[Source::new(Located::new("a.rs"), Text::new(IF_IN_ARM))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("loops"),
        &[Source::new(Located::new("a.rs"), Text::new(LOOPS))],
        Flagged::new(&["a.rs:2", "a.rs:3", "a.rs:4", "a.rs:5"]),
    ),
    Case::new(
        Title::new("let-else"),
        &[Source::new(Located::new("a.rs"), Text::new(LET_ELSE))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("match in a closure"),
        &[Source::new(Located::new("a.rs"), Text::new(CLOSURE))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(Title::new("const fn"), &[Source::new(Located::new("a.rs"), Text::new(CONST_FN))], Flagged::new(&[])),
    Case::new(
        Title::new("nested fn judged alone"),
        &[Source::new(Located::new("a.rs"), Text::new(NESTED))],
        Flagged::new(&["a.rs:9"]),
    ),
    Case::new(
        Title::new("test module"),
        &[Source::new(Located::new("a.rs"), Text::new(TEST_MODULE))],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(
        Title::new("integration test"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(INTEGRATION))],
        Flagged::new(&["tests/a.rs:3"]),
    ),
    Case::new(
        Title::new("inside a macro"),
        &[Source::new(Located::new("a.rs"), Text::new(IN_MACRO))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("fn inside macro_rules"),
        &[Source::new(Located::new("a.rs"), Text::new(IN_MACRO_RULES))],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(
        Title::new("trait default and impl"),
        &[Source::new(Located::new("a.rs"), Text::new(METHODS))],
        Flagged::new(&["a.rs:3", "a.rs:8"]),
    ),
    Case::new(
        Title::new("from bool in a boundary module file"),
        &[Source::new(Located::new("crates/x/src/boundary/lit.rs"), Text::new(FROM_BOOL)), TYPES],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("from bool in an inline boundary module"),
        &[Source::new(Located::new("crates/x/src/lib.rs"), Text::new(FROM_BOOL_INLINE)), TYPES],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("from bool outside a boundary module"),
        &[Source::new(Located::new("crates/x/src/lit.rs"), Text::new(FROM_BOOL)), TYPES],
        Flagged::new(&["crates/x/src/lit.rs:3"]),
    ),
    Case::new(
        Title::new("from bool with more than the if"),
        &[Source::new(Located::new("crates/x/src/boundary/lit.rs"), Text::new(FROM_BOOL_AFTER_LET)), TYPES],
        Flagged::new(&["crates/x/src/boundary/lit.rs:4"]),
    ),
    Case::new(
        Title::new("from bool deciding on more than its value"),
        &[Source::new(Located::new("crates/x/src/boundary/lit.rs"), Text::new(FROM_BOOL_JOINED)), TYPES],
        Flagged::new(&["crates/x/src/boundary/lit.rs:3"]),
    ),
    Case::new(
        Title::new("from bool with an else if"),
        &[Source::new(Located::new("crates/x/src/boundary/lit.rs"), Text::new(FROM_BOOL_ELSE_IF)), TYPES],
        Flagged::new(&["crates/x/src/boundary/lit.rs:3", "crates/x/src/boundary/lit.rs:3"]),
    ),
    Case::new(
        Title::new("from bool with an if inside a branch"),
        &[Source::new(Located::new("crates/x/src/boundary/lit.rs"), Text::new(FROM_BOOL_NESTED)), TYPES],
        Flagged::new(&["crates/x/src/boundary/lit.rs:3", "crates/x/src/boundary/lit.rs:3"]),
    ),
    Case::new(
        Title::new("a helper beside from bool"),
        &[Source::new(Located::new("crates/x/src/boundary/lit.rs"), Text::new(FROM_BOOL_HELPER)), TYPES],
        Flagged::new(&["crates/x/src/boundary/lit.rs:8"]),
    ),
    Case::new(
        Title::new("from bool deciding on a constant"),
        &[Source::new(Located::new("crates/x/src/boundary/lit.rs"), Text::new(FROM_BOOL_CONST)), TYPES],
        Flagged::new(&["crates/x/src/boundary/lit.rs:3"]),
    ),
    Case::new(
        Title::new("a function nested in from bool"),
        &[Source::new(Located::new("crates/x/src/boundary/lit.rs"), Text::new(FROM_BOOL_PICK)), TYPES],
        Flagged::new(&["crates/x/src/boundary/lit.rs:3"]),
    ),
    Case::new(
        Title::new("from another type in a boundary module"),
        &[Source::new(Located::new("crates/x/src/boundary/lit.rs"), Text::new(FROM_COUNT))],
        Flagged::new(&["crates/x/src/boundary/lit.rs:3"]),
    ),
    Case::new(
        Title::new("from bool into a struct built by calls"),
        &[Source::new(Located::new("crates/x/src/boundary/lit.rs"), Text::new(FROM_BOOL_STRUCT))],
        Flagged::new(&["crates/x/src/boundary/lit.rs:6"]),
    ),
    Case::new(
        Title::new("from bool with a compound branch"),
        &[Source::new(Located::new("crates/x/src/boundary/lit.rs"), Text::new(FROM_BOOL_COMPOUND)), TYPES],
        Flagged::new(&["crates/x/src/boundary/lit.rs:3"]),
    ),
    Case::new(
        Title::new("a function nested in an associated const of from bool"),
        &[Source::new(Located::new("crates/x/src/boundary/lit.rs"), Text::new(FROM_BOOL_CONST_FN)), TYPES],
        Flagged::new(&["crates/x/src/boundary/lit.rs:3"]),
    ),
    Case::new(
        Title::new("from bool naming Self against the type"),
        &[Source::new(Located::new("crates/x/src/boundary/lit.rs"), Text::new(FROM_BOOL_SELF)), TYPES],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("from bool choosing module constants"),
        &[Source::new(Located::new("crates/x/src/boundary/lit.rs"), Text::new(FROM_BOOL_MODULE)), TYPES],
        Flagged::new(&["crates/x/src/boundary/lit.rs:3"]),
    ),
    Case::new(
        Title::new("from bool choosing associated constants"),
        &[Source::new(Located::new("crates/x/src/boundary/lit.rs"), Text::new(FROM_BOOL_LIMIT)), TYPES],
        Flagged::new(&["crates/x/src/boundary/lit.rs:3"]),
    ),
    Case::new(
        Title::new("from bool choosing capital and digit variants"),
        &[Source::new(Located::new("crates/x/src/boundary/family.rs"), Text::new(FROM_BOOL_FAMILY)), TYPES],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("from bool choosing lowercase names"),
        &[Source::new(Located::new("crates/x/src/boundary/lit.rs"), Text::new(FROM_BOOL_LOWER)), TYPES],
        Flagged::new(&["crates/x/src/boundary/lit.rs:3"]),
    ),
    Case::new(
        Title::new("from bool with identical branches"),
        &[Source::new(Located::new("crates/x/src/boundary/lit.rs"), Text::new(FROM_BOOL_SAME)), TYPES],
        Flagged::new(&["crates/x/src/boundary/lit.rs:3"]),
    ),
    Case::new(
        Title::new("from bool choosing single letter variants"),
        &[Source::new(Located::new("crates/x/src/boundary/side.rs"), Text::new(FROM_BOOL_SIDE)), TYPES],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("from bool in a ports module file"),
        &[Source::new(Located::new("crates/x/src/ports/lit.rs"), Text::new(FROM_BOOL)), TYPES],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("from bool choosing variants of another type"),
        &[Source::new(Located::new("crates/x/src/boundary/lit.rs"), Text::new(FROM_BOOL_FOREIGN)), TYPES],
        Flagged::new(&["crates/x/src/boundary/lit.rs:3"]),
    ),
    Case::new(
        Title::new("from bool in an inline ports module"),
        &[Source::new(Located::new("crates/x/src/lib.rs"), Text::new(FROM_BOOL_PORTS_INLINE)), TYPES],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("from bool taking two inputs"),
        &[Source::new(Located::new("crates/x/src/boundary/lit.rs"), Text::new(FROM_BOOL_TWO_INPUTS)), TYPES],
        Flagged::new(&["crates/x/src/boundary/lit.rs:3"]),
    ),
    Case::new(
        Title::new("from bool choosing constants of a struct"),
        &[Source::new(Located::new("crates/x/src/boundary/lit.rs"), Text::new(FROM_BOOL_ONOFF)), STRUCT_LIT],
        Flagged::new(&["crates/x/src/boundary/lit.rs:3"]),
    ),
    Case::new(
        Title::new("from bool choosing single word constants of a struct"),
        &[Source::new(Located::new("crates/x/src/boundary/delay.rs"), Text::new(FROM_BOOL_DELAY)), STRUCT_DELAY],
        Flagged::new(&["crates/x/src/boundary/delay.rs:3"]),
    ),
    Case::new(
        Title::new("from bool choosing a name the enum does not declare"),
        &[Source::new(Located::new("crates/x/src/boundary/lit.rs"), Text::new(FROM_BOOL_GONE)), TYPES],
        Flagged::new(&["crates/x/src/boundary/lit.rs:3"]),
    ),
    Case::new(
        Title::new("from bool into an enum declared twice, both declaring the chosen variants"),
        &[Source::new(Located::new("crates/x/src/boundary/lit.rs"), Text::new(FROM_BOOL)), TYPES, TWIN_LIT],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("from bool into an enum sharing its name with a struct"),
        &[Source::new(Located::new("crates/x/src/boundary/lit.rs"), Text::new(FROM_BOOL)), TYPES, STRUCT_TWIN_LIT],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("from bool into an enum twin lacking a chosen variant"),
        &[Source::new(Located::new("crates/x/src/boundary/lit.rs"), Text::new(FROM_BOOL)), TYPES, PARTIAL_TWIN_LIT],
        Flagged::new(&["crates/x/src/boundary/lit.rs:3"]),
    ),
    Case::new(
        Title::new("from bool into an enum declared in a test module"),
        &[Source::new(Located::new("crates/x/src/boundary/lit.rs"), Text::new(FROM_BOOL_IN_TESTS))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("from bool into an enum made by a macro is flagged, as macro expansion is not read"),
        &[Source::new(Located::new("crates/x/src/boundary/lit.rs"), Text::new(FROM_BOOL)), MACRO_LIT],
        Flagged::new(&["crates/x/src/boundary/lit.rs:3"]),
    ),
    Case::new(
        Title::new("from bool into an enum split by cfg"),
        &[Source::new(Located::new("crates/x/src/boundary/lit.rs"), Text::new(FROM_BOOL)), CFG_TWIN_LIT],
        Flagged::new(&[]),
    ),
];

const ROUTERS: [Case; 13] = [
    Case::new(
        Title::new("from bool matched on true and false"),
        &[Source::new(Located::new("crates/x/src/boundary/lit.rs"), Text::new(FROM_BOOL_MATCHED))],
        Flagged::new(&["crates/x/src/boundary/lit.rs:4", "crates/x/src/boundary/lit.rs:5"]),
    ),
    Case::new(Title::new("pure router"), &[Source::new(Located::new("a.rs"), Text::new(ROUTER))], Flagged::new(&[])),
    Case::new(Title::new("pipeline"), &[Source::new(Located::new("a.rs"), Text::new(PIPELINE))], Flagged::new(&[])),
    Case::new(Title::new("guard"), &[Source::new(Located::new("a.rs"), Text::new(GUARD))], Flagged::new(&["a.rs:3"])),
    Case::new(
        Title::new("literal and range"),
        &[Source::new(Located::new("a.rs"), Text::new(LITERAL))],
        Flagged::new(&["a.rs:3", "a.rs:4"]),
    ),
    Case::new(
        Title::new("boolean"),
        &[Source::new(Located::new("a.rs"), Text::new(BOOLEAN))],
        Flagged::new(&["a.rs:3", "a.rs:4"]),
    ),
    Case::new(Title::new("tuple"), &[Source::new(Located::new("a.rs"), Text::new(TUPLE))], Flagged::new(&["a.rs:3"])),
    Case::new(
        Title::new("nested patterns"),
        &[Source::new(Located::new("a.rs"), Text::new(NESTED_PATTERNS))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("arm shapes"),
        &[Source::new(Located::new("a.rs"), Text::new(ARMS))],
        Flagged::new(&["a.rs:3", "a.rs:7"]),
    ),
    Case::new(
        Title::new("else if"),
        &[Source::new(Located::new("a.rs"), Text::new(ELSE_IF))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("if let literal with logic"),
        &[Source::new(Located::new("a.rs"), Text::new(IF_LET_LITERAL))],
        Flagged::new(&["a.rs:2", "a.rs:2"]),
    ),
    Case::new(
        Title::new("wrapped literals"),
        &[Source::new(Located::new("a.rs"), Text::new(WRAPPED))],
        Flagged::new(&["a.rs:3", "a.rs:4", "a.rs:5", "a.rs:6"]),
    ),
    Case::new(
        Title::new("guard outside a router"),
        &[Source::new(Located::new("a.rs"), Text::new(GUARD_OUTSIDE))],
        Flagged::new(&[]),
    ),
];

const CONDITIONS: [Case; 3] = [
    Case::new(
        Title::new("operators"),
        &[Source::new(Located::new("a.rs"), Text::new(OPERATORS))],
        Flagged::new(&["a.rs:2", "a.rs:3", "a.rs:3", "a.rs:4"]),
    ),
    Case::new(Title::new("pipeline"), &[Source::new(Located::new("a.rs"), Text::new(PIPELINE))], Flagged::new(&[])),
    Case::new(
        Title::new("test module"),
        &[Source::new(Located::new("a.rs"), Text::new(OPERATOR_IN_TEST))],
        Flagged::new(&["a.rs:4"]),
    ),
];

const IGNORED: &str =
    "fn f(items: &[Lamp], ready: Ready) -> usize {\n    items.iter().filter(|_| ready.now()).count()\n}\n";
const UNREAD: &str =
    "fn f(items: &[Lamp], ready: Ready) -> usize {\n    items.iter().filter(|lamp| ready.now()).count()\n}\n";
const SINGLE: &str = "fn f(lamp: Lamp, ready: Ready) -> Option<Lamp> {\n    Some(lamp).filter(|_| ready.now())\n}\n";
const QUANTIFIED: &str = "fn f(items: &[Lamp], ready: Ready) -> Lit {\n    let any = items.iter().any(|_| ready.now());\n    let all = items.iter().all(|_| ready.now());\n    let at = items.iter().position(|_| ready.now());\n    Lit::new(any, all, at)\n}\n";
const READ: &str = "fn f(items: &[Lamp]) -> usize {\n    items.iter().filter(|lamp| lamp.on()).count()\n}\n";
const READ_IN_MACRO: &str =
    "fn f(items: &[Lamp]) -> usize {\n    items.iter().filter(|lamp| matches!(lamp, Lamp::On)).count()\n}\n";
const DESTRUCTURED: &str =
    "fn f(items: &[Lamp]) -> usize {\n    items.iter().filter(|&&Lamp { on, .. }| on).count()\n}\n";
const NAMED_PREDICATE: &str =
    "fn f(items: &[Lamp]) -> usize {\n    items.iter().copied().filter(Lamp::on).count()\n}\n";
const OTHER_METHOD: &str =
    "fn f(items: &[Lamp], ready: Ready) -> Option<&Lamp> {\n    items.iter().find(|_| ready.now())\n}\n";
const SELECTION_IN_TEST: &str = "#[cfg(test)]\nmod tests {\n    fn f(items: &[u8]) -> usize {\n        items.iter().filter(|_| READY).count()\n    }\n}\n";

const NESTED_BINDING: &str = "fn f(items: &[Lamp], flags: &[Flag]) -> usize {\n    items.iter().filter(|_| flags.iter().any(|flag| flag.on())).count()\n}\n";
const LET_BINDING: &str = "fn f(items: &[Lamp], ready: Ready) -> usize {\n    items.iter().filter(|_| { let now = ready.now(); now }).count()\n}\n";
const SHADOWED_ELEMENT: &str = "fn f(items: &[Lamp], others: &[Lamp]) -> usize {\n    items.iter().filter(|lamp| others.iter().any(|lamp| lamp.on())).count()\n}\n";
const INLINE_FORMAT: &str = "fn f(items: &[Lamp], seen: &mut Seen) -> usize {\n    items.iter().filter(|lamp| seen.insert(format!(\"{lamp}\"))).count()\n}\n";

const SHADOW_CLOSURE: &str = "fn f(items: &[Lamp], others: &[Lamp]) -> usize {\n    items.iter().filter(|lamp| others.iter().any(|lamp| lamp.on())).count()\n}\n";
const READ_CLOSURE: &str = "fn f(items: &[Lamp], others: &[Lamp]) -> usize {\n    items.iter().filter(|lamp| others.iter().any(|other| other.sees(lamp))).count()\n}\n";
const SHADOW_BLOCK: &str = "fn f(items: &[Lamp], others: &[Lamp]) -> usize {\n    items.iter().filter(|lamp| { let lamp = others.first(); lamp.on() }).count()\n}\n";
const READ_BLOCK: &str = "fn f(items: &[Lamp], others: &[Lamp]) -> usize {\n    items.iter().filter(|lamp| { let seen = others.first(); seen.sees(lamp) }).count()\n}\n";
const SHADOW_ARM: &str = "fn f(items: &[Lamp], others: &[Lamp]) -> usize {\n    items.iter().filter(|lamp| match others.first() { Some(lamp) => lamp.on(), None => false }).count()\n}\n";
const READ_ARM: &str = "fn f(items: &[Lamp], others: &[Lamp]) -> usize {\n    items.iter().filter(|lamp| match others.first() { Some(other) => other.sees(lamp), None => false }).count()\n}\n";
const SHADOW_IF_LET: &str = "fn f(items: &[Lamp], others: &[Lamp]) -> usize {\n    items.iter().filter(|lamp| if let Some(lamp) = others.first() { lamp.on() } else { false }).count()\n}\n";
const READ_IF_LET: &str = "fn f(items: &[Lamp], others: &[Lamp]) -> usize {\n    items.iter().filter(|lamp| if let Some(other) = others.first() { other.sees(lamp) } else { false }).count()\n}\n";
const SHADOW_FOR: &str = "fn f(items: &[Lamp], others: &[Lamp]) -> usize {\n    items.iter().filter(|lamp| { for lamp in others { lamp.on(); } true }).count()\n}\n";
const READ_FOR: &str = "fn f(items: &[Lamp], others: &[Lamp]) -> usize {\n    items.iter().filter(|lamp| { for other in others { other.sees(lamp); } true }).count()\n}\n";
const READ_IF: &str = "fn f(items: &[Lamp], others: &[Lamp]) -> usize {\n    items.iter().filter(|lamp| if ready() { lamp.on() } else { false }).count()\n}\n";

const SELECTIONS: [Case; 25] = [
    Case::new(
        Title::new("filter ignoring its element"),
        &[Source::new(Located::new("a.rs"), Text::new(IGNORED))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("filter naming its element and not reading it"),
        &[Source::new(Located::new("a.rs"), Text::new(UNREAD))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("filter on a single value"),
        &[Source::new(Located::new("a.rs"), Text::new(SINGLE))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("any, all and position ignoring their element"),
        &[Source::new(Located::new("a.rs"), Text::new(QUANTIFIED))],
        Flagged::new(&["a.rs:2", "a.rs:3", "a.rs:4"]),
    ),
    Case::new(
        Title::new("filter reading its element"),
        &[Source::new(Located::new("a.rs"), Text::new(READ))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("filter reading its element inside a macro"),
        &[Source::new(Located::new("a.rs"), Text::new(READ_IN_MACRO))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("filter reading a destructured field"),
        &[Source::new(Located::new("a.rs"), Text::new(DESTRUCTURED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("filter by a named predicate"),
        &[Source::new(Located::new("a.rs"), Text::new(NAMED_PREDICATE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a method outside the selections"),
        &[Source::new(Located::new("a.rs"), Text::new(OTHER_METHOD))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("selection in a test module"),
        &[Source::new(Located::new("a.rs"), Text::new(SELECTION_IN_TEST))],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(
        Title::new("nested closure binding is not a read"),
        &[Source::new(Located::new("a.rs"), Text::new(NESTED_BINDING))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("let binding in the body is not a read"),
        &[Source::new(Located::new("a.rs"), Text::new(LET_BINDING))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("shadowed element is not a read"),
        &[Source::new(Located::new("a.rs"), Text::new(SHADOWED_ELEMENT))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("inline format capture reads the element"),
        &[Source::new(Located::new("a.rs"), Text::new(INLINE_FORMAT))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("closure param shadowing the element is not a read"),
        &[Source::new(Located::new("a.rs"), Text::new(SHADOW_CLOSURE))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("closure param reading the element"),
        &[Source::new(Located::new("a.rs"), Text::new(READ_CLOSURE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("block let shadowing the element is not a read"),
        &[Source::new(Located::new("a.rs"), Text::new(SHADOW_BLOCK))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("block let reading the element"),
        &[Source::new(Located::new("a.rs"), Text::new(READ_BLOCK))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("match arm shadowing the element is not a read"),
        &[Source::new(Located::new("a.rs"), Text::new(SHADOW_ARM))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("match arm reading the element"),
        &[Source::new(Located::new("a.rs"), Text::new(READ_ARM))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("if let shadowing the element is not a read"),
        &[Source::new(Located::new("a.rs"), Text::new(SHADOW_IF_LET))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("if let reading the element"),
        &[Source::new(Located::new("a.rs"), Text::new(READ_IF_LET))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("for pattern shadowing the element is not a read"),
        &[Source::new(Located::new("a.rs"), Text::new(SHADOW_FOR))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("for pattern reading the element"),
        &[Source::new(Located::new("a.rs"), Text::new(READ_FOR))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("plain if reading the element"),
        &[Source::new(Located::new("a.rs"), Text::new(READ_IF))],
        Flagged::new(&[]),
    ),
];

#[test]
fn bodies_with_if_match_or_loops_outside_a_router_are_flagged() {
    assert_eq!(Sources::mismatches::<LinearBodies>(&LINEAR), Lines::default());
}

#[test]
fn routers_with_guards_value_patterns_or_logic_in_arms_are_flagged() {
    assert_eq!(Sources::mismatches::<PureRouters>(&ROUTERS), Lines::default());
}

#[test]
fn boolean_operators_are_flagged_in_shipped_and_test_code() {
    assert_eq!(Sources::mismatches::<PlainConditions>(&CONDITIONS), Lines::default());
}

#[test]
fn selections_whose_predicate_ignores_its_element_are_flagged() {
    assert_eq!(Sources::mismatches::<IndependentSelections>(&SELECTIONS), Lines::default());
}

#[test]
fn each_crate_counts_its_pipelines_routers_and_the_rest() {
    let files = Sources::parsed(&[Source::new(Located::new("crates/x/src/lib.rs"), Text::new(SHAPES))]);
    let shown: Vec<String> = FlowShapes::report(&files).violations().iter().map(ToString::to_string).collect();
    assert_eq!(
        shown,
        [
            "crates/x/src/lib.rs:3: workspace: 3 functions, 1 pipelines, 1 pure routers, 1 neither; turn each function that is neither into a pure router (one match on an enum, Option or Result), or give its decisions a named outcome (an enum a pure router matches)",
            "crates/x/src/lib.rs:3: crate x: 3 functions, 1 pipelines, 1 pure routers, 1 neither; turn each function that is neither into a pure router (one match on an enum, Option or Result), or give its decisions a named outcome (an enum a pure router matches)",
        ]
    );
}

#[test]
fn messages_name_the_branch_and_its_fix() {
    let files = Sources::parsed(&[
        Source::new(Located::new("a.rs"), Text::new(IF)),
        Source::new(Located::new("b.rs"), Text::new(IF_LET_INSIDE)),
        Source::new(Located::new("c.rs"), Text::new(MATCH_AFTER_LET)),
        Source::new(Located::new("d.rs"), Text::new(LET_ELSE)),
    ]);
    let shown: Vec<String> = LinearBodies::report(&files).violations().iter().map(ToString::to_string).collect();
    assert_eq!(
        shown,
        [
            "a.rs:2: if; turn the whole body into a pure router (one match on an enum, Option or Result), or give the decision a named outcome (an enum a pure router matches)",
            "b.rs:3: if let that is not the whole body; extract the if let into a pure router (a function whose whole body is that if let), or give the decision a named outcome (an enum a pure router matches)",
            "c.rs:3: match that is not the whole body; extract the match into a pure router (a function whose whole body is that match), or give the decision a named outcome (an enum a pure router matches)",
            "d.rs:2: let-else; extract a pure router (a function whose whole body is one match on the Option or Result), or give the decision a named outcome (an enum a pure router matches)",
        ]
    );
}

#[test]
fn loops_routers_and_conditions_pin_their_fix_text() {
    let files = Sources::parsed(&[
        Source::new(Located::new("e.rs"), Text::new(LOOPS)),
        Source::new(Located::new("f.rs"), Text::new(GUARD)),
        Source::new(Located::new("g.rs"), Text::new(LITERAL)),
        Source::new(Located::new("h.rs"), Text::new(ARMS)),
        Source::new(Located::new("i.rs"), Text::new(OPERATORS)),
    ]);
    let shown: Vec<String> = LinearBodies::report(&files)
        .violations()
        .iter()
        .chain(PureRouters::report(&files).violations().iter())
        .chain(PlainConditions::report(&files).violations().iter())
        .map(ToString::to_string)
        .collect();
    assert_eq!(
        shown,
        [
            "e.rs:2: for loop; drive an iterator with combinators (map, fold, try_fold, take_while, or filter on a predicate that reads its element)",
            "e.rs:3: while loop; drive an iterator with combinators (map, fold, try_fold, take_while, or filter on a predicate that reads its element)",
            "e.rs:4: while loop; drive an iterator with combinators (map, fold, try_fold, take_while, or filter on a predicate that reads its element)",
            "e.rs:5: loop; drive an iterator with combinators (successors, repeat_with, take_while, try_fold)",
            "f.rs:3: match guard in a router; name the condition as an enum or a predicate type and match on it",
            "g.rs:3: router arm matches a literal, range, tuple or slice; match on an enum, Option or Result that names the cases",
            "g.rs:4: router arm matches a literal, range, tuple or slice; match on an enum, Option or Result that names the cases",
            "h.rs:3: router arm is not a single call or value; extract the arm's logic into its own linear function and call it",
            "h.rs:7: router arm is not a single call or value; extract the arm's logic into its own linear function and call it",
            "i.rs:2: boolean operator &&; name the condition as an enum or a predicate type",
            "i.rs:3: boolean operator ||; name the condition as an enum or a predicate type",
            "i.rs:3: boolean operator !; name the condition as an enum or a predicate type",
            "i.rs:4: boolean operator &&; name the condition as an enum or a predicate type",
        ]
    );
}
