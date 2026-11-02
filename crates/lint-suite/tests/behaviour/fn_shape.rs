use lint_fn_shape::FnShape;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    fn f(a: u8, b: u8, c: u8, d: u8, e: u8) {}\n}\n";

const TEST_IMPL: &str = "struct A;\n#[cfg(test)]\nimpl A {\n    fn f(a: u8, b: u8, c: u8, d: u8, e: u8) {}\n}\n";
const TEST_FN: &str = "#[cfg(test)]\nfn f(a: u8, b: u8, c: u8, d: u8, e: u8) {}\n";
const GENERATED: &str = "struct A;\nmacro_rules! wide {\n    () => {\n        impl A {\n            fn f(a: u8, b: u8, c: u8, d: u8, e: u8) {}\n        }\n    };\n}\n";

const INLINE: &str = "mod inner {\n    pub fn f(a: u8, b: u8, c: u8, d: u8, e: u8) {}\n}\n";
const DEFAULT_METHOD: &str = "trait T {\n    fn f(a: u8, b: u8, c: u8, d: u8, e: u8) {}\n}\n";
const TRY_FOLD: &str = "fn f() {\n    let total = items.iter().try_fold((0, 0), |a, b| a);\n}\n";
const ARMS: &str = "fn f(a: u8) {\n    match a {\n        _ => match a {\n            _ => match a {\n                _ => g(),\n            },\n        },\n    }\n}\n";
const FORS: &str =
    "fn f(a: u8) {\n    for _ in a {\n        for _ in a {\n            for _ in a {}\n        }\n    }\n}\n";
const WHILES: &str = "fn f(a: u8) {\n    while a {\n        while a {\n            while a {}\n        }\n    }\n}\n";
const LOOPS: &str = "fn f(a: u8) {\n    loop {\n        loop {\n            loop {}\n        }\n    }\n}\n";
const CLOSURES: &str = "fn f() {\n    a(|| {\n        a(|| {\n            a(|| g());\n        });\n    });\n}\n";
const SIBLINGS: &str = "fn f(a: bool) {\n    if a {\n        if a {\n            g();\n        }\n    }\n    if a {\n        if a {\n            g();\n        }\n    }\n}\n";
const BOOLEANS: &str = "fn f(x: u8) {\n    let a = !b;\n    let c = true;\n    let d = (x > 1);\n    let e = x.is_empty();\n    let h = x.contains(1);\n    let i = x.len();\n}\n";

const CASES: [Case; 14] = [
    Case::new(
        Title::new("test module"),
        &[Source::new(Located::new("a.rs"), Text::new(TEST_MODULE))],
        Flagged::new(&[]),
    ),
    Case::new(Title::new("test impl"), &[Source::new(Located::new("a.rs"), Text::new(TEST_IMPL))], Flagged::new(&[])),
    Case::new(Title::new("test fn"), &[Source::new(Located::new("a.rs"), Text::new(TEST_FN))], Flagged::new(&[])),
    Case::new(
        Title::new("inside a macro"),
        &[Source::new(Located::new("a.rs"), Text::new(GENERATED))],
        Flagged::new(&["a.rs:5"]),
    ),
    Case::new(
        Title::new("inline module"),
        &[Source::new(Located::new("a.rs"), Text::new(INLINE))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("default method"),
        &[Source::new(Located::new("a.rs"), Text::new(DEFAULT_METHOD))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("try_fold seed"),
        &[Source::new(Located::new("a.rs"), Text::new(TRY_FOLD))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("match arms nest"),
        &[Source::new(Located::new("a.rs"), Text::new(ARMS))],
        Flagged::new(&["a.rs:5"]),
    ),
    Case::new(
        Title::new("for loops nest"),
        &[Source::new(Located::new("a.rs"), Text::new(FORS))],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(
        Title::new("while loops nest"),
        &[Source::new(Located::new("a.rs"), Text::new(WHILES))],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(
        Title::new("loops nest"),
        &[Source::new(Located::new("a.rs"), Text::new(LOOPS))],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(
        Title::new("closures nest"),
        &[Source::new(Located::new("a.rs"), Text::new(CLOSURES))],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(
        Title::new("sibling blocks do not add up"),
        &[Source::new(Located::new("a.rs"), Text::new(SIBLINGS))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("each boolean form"),
        &[Source::new(Located::new("a.rs"), Text::new(BOOLEANS))],
        Flagged::new(&["a.rs:2", "a.rs:3", "a.rs:4", "a.rs:5", "a.rs:6"]),
    ),
];

#[test]
fn hairy_control_flow_is_flagged() {
    assert_eq!(Sources::mismatches::<FnShape>(&CASES), Lines::default());
}
