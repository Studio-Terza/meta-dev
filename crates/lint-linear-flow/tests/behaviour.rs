use lint_core::Lint;
use lint_file::Filed;
use lint_linear_flow::{FlowShapes, LinearBodies, PlainConditions, PureRouters};

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

const SHAPES: &str = "struct A;\nimpl A {\n    fn piped(&self) -> u8 {\n        g().h()\n    }\n    fn routed(&self, on: Switch) -> u8 {\n        match on {\n            Switch::On => 1,\n            Switch::Off => 0,\n        }\n    }\n    fn tangled(&self) -> u8 {\n        if p { 1 } else { 0 }\n    }\n    const fn fixed() -> u8 {\n        if p { 1 } else { 0 }\n    }\n}\n";

const LINEAR: [Case; 17] = [
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
];

const ROUTERS: [Case; 12] = [
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

#[test]
fn bodies_with_if_match_or_loops_outside_a_router_are_flagged() {
    assert_eq!(Sources::mismatches::<Filed<LinearBodies>>(&LINEAR), Lines::default());
}

#[test]
fn routers_with_guards_value_patterns_or_logic_in_arms_are_flagged() {
    assert_eq!(Sources::mismatches::<Filed<PureRouters>>(&ROUTERS), Lines::default());
}

#[test]
fn boolean_operators_are_flagged_in_shipped_and_test_code() {
    assert_eq!(Sources::mismatches::<Filed<PlainConditions>>(&CONDITIONS), Lines::default());
}

#[test]
fn each_crate_counts_its_pipelines_routers_and_the_rest() {
    let files = Sources::parsed(&[Source::new(Located::new("crates/x/src/lib.rs"), Text::new(SHAPES))]);
    let shown: Vec<String> = FlowShapes::report(&files).violations().iter().map(ToString::to_string).collect();
    assert_eq!(
        shown,
        [
            "crates/x/src/lib.rs:3: workspace: 3 functions, 1 pipelines, 1 pure routers, 1 neither; turn each function that is neither into a pipeline of let bindings and combinators, or into a pure router",
            "crates/x/src/lib.rs:3: crate x: 3 functions, 1 pipelines, 1 pure routers, 1 neither; turn each function that is neither into a pipeline of let bindings and combinators, or into a pure router",
        ]
    );
}

#[test]
fn messages_name_the_branch_and_its_fix() {
    let files = Sources::parsed(&[
        Source::new(Located::new("a.rs"), Text::new(IF)),
        Source::new(Located::new("b.rs"), Text::new(IF_LET_INSIDE)),
    ]);
    let shown: Vec<String> =
        Filed::<LinearBodies>::report(&files).violations().iter().map(ToString::to_string).collect();
    assert_eq!(
        shown,
        [
            "a.rs:2: if; use combinators on Option, Result and iterators (then_some, filter, map_or), or make the whole body one match on an enum, Option or Result",
            "b.rs:3: if let that is not the whole body; use combinators on Option and Result (map, and_then, map_or), or extract it into its own function whose whole body is the if let",
        ]
    );
}
