use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};
use lint_repeated_bodies::RepeatedBodies;

const BODY: &str = "fn a(items: &[u32]) -> u32 {\n    let mut total = 0;\n    for item in items {\n        if *item > 3 {\n            total += item * 2;\n        } else {\n            total += 1;\n        }\n    }\n    if total > 100 {\n        return 100;\n    }\n    total\n}\n";
const RENAMED: &str = "fn b(rows: &[u32]) -> u32 {\n    let mut sum = 10;\n    for row in rows {\n        if *row > 7 {\n            sum += row * 5;\n        } else {\n            sum += 9;\n        }\n    }\n    if sum > 100 {\n        return 100;\n    }\n    sum\n}\n";
const OTHER_CALL: &str = "fn c(items: &[u32]) -> u32 {\n    let mut total = 0;\n    for item in items {\n        if keeps(*item) {\n            total += item * 2;\n        } else {\n            total += 1;\n        }\n    }\n    if total > 100 {\n        return 100;\n    }\n    total\n}\n";
const SHORT_A: &str = "fn a() {\n    f(1);\n}\n";
const SHORT_B: &str = "fn b() {\n    f(2);\n}\n";
const TESTS_A: &str = "#[cfg(test)]\nmod tests {\n    fn a(items: &[u32]) -> u32 {\n        let mut total = 0;\n        for item in items {\n            if *item > 3 {\n                total += item * 2;\n            } else {\n                total += 1;\n            }\n        }\n        total\n    }\n}\n";
const CONST_A: &str = "const fn a(items: &[u32]) -> u32 {\n    let mut total = 0;\n    let mut at = 0;\n    while at < items.len() {\n        if items[at] > 3 {\n            total += items[at] * 2;\n        } else {\n            total += 1;\n        }\n        at += 1;\n    }\n    if total > 100 {\n        return 100;\n    }\n    total\n}\n";
const CONST_B: &str = "const fn b(rows: &[u32]) -> u32 {\n    let mut sum = 0;\n    let mut at = 0;\n    while at < rows.len() {\n        if rows[at] > 3 {\n            sum += rows[at] * 2;\n        } else {\n            sum += 1;\n        }\n        at += 1;\n    }\n    if sum > 100 {\n        return 100;\n    }\n    sum\n}\n";

const THREE: &str = "fn a(items: &[u32]) -> u32 {\n    let mut total = 0;\n    for item in items {\n        if *item > 3 {\n            total += item * 2;\n        } else {\n            total += 1;\n        }\n    }\n    if total > 100 {\n        return 100;\n    }\n    total\n}\nfn b(items: &[u32]) -> u32 {\n    let mut total = 0;\n    for item in items {\n        if *item > 4 {\n            total += item * 3;\n        } else {\n            total += 2;\n        }\n    }\n    if total > 100 {\n        return 100;\n    }\n    total\n}\nfn c(items: &[u32]) -> u32 {\n    let mut total = 0;\n    for item in items {\n        if *item > 5 {\n            total += item * 4;\n        } else {\n            total += 3;\n        }\n    }\n    if total > 100 {\n        return 100;\n    }\n    total\n}\n";
const NESTED_A: &str = "fn a(x: u8) -> String {\n    let label = match x {\n        0 => format!(\"zero {}\", x),\n        1 => format!(\"one {}\", x),\n        2 => format!(\"two {}\", x),\n        _ => format!(\"many {}\", x),\n    };\n    label\n}\n";
const NESTED_B: &str = "fn b(y: u8) -> String {\n    let text = match y {\n        0 => format!(\"zero {}\", y),\n        1 => format!(\"one {}\", y),\n        2 => format!(\"two {}\", y),\n        _ => format!(\"many {}\", y),\n    };\n    text\n}\n";

const MATCH_A: &str = "fn a(x: u8) -> String {\n    let first = 1;\n    let label = match x {\n        0 => format!(\"zero {}\", x),\n        1 => format!(\"one {}\", x),\n        2 => format!(\"two {}\", x),\n        _ => format!(\"many {}\", x),\n    };\n    label\n}\n";
const MATCH_B: &str = "fn b(y: u8) -> String {\n    let text = match y {\n        0 => format!(\"zero {}\", y),\n        1 => format!(\"one {}\", y),\n        2 => format!(\"two {}\", y),\n        _ => format!(\"many {}\", y),\n    };\n    other(text)\n}\n";
const CLOSURE_A: &str = "fn a(x: u8) -> u8 {\n    let first = 1;\n    let g = |y: u8|\n    {\n        if y > 3 { return y * 2; }\n        if y > 5 { return y * 3; }\n        if y > 7 { return y * 4; }\n        if y > 9 { return y * 5; }\n        y\n    };\n    g(x)\n}\n";
const CLOSURE_B: &str = "fn b(x: u8) -> u8 {\n    let g = |z: u8|\n    {\n        if z > 3 { return z * 2; }\n        if z > 5 { return z * 3; }\n        if z > 7 { return z * 4; }\n        if z > 9 { return z * 5; }\n        z\n    };\n    g(x) + 1\n}\n";
const CHAIN_A: &str = "fn a(x: Q) -> R {\n    let first = 1;\n    let r = x\n        .alpha(1).beta(2).gamma(3).delta(4).epsilon(5).zeta(6).eta(7).theta(8).iota(9);\n    r\n}\n";
const CHAIN_B: &str = "fn b(y: Q) -> R {\n    let r = y\n        .alpha(1).beta(2).gamma(3).delta(4).epsilon(5).zeta(6).eta(7).theta(8).iota(9);\n    other(r)\n}\n";
const TRAITS: &str = "trait T {\n    fn declared(&self);\n    fn a(&self, items: &[u32]) -> u32 {\n        let mut total = 0;\n        for item in items {\n            if *item > 3 {\n                total += item * 2;\n            } else {\n                total += 1;\n            }\n        }\n        if total > 100 {\n            return 100;\n        }\n        total\n    }\n}\ntrait U {\n    fn b(&self, rows: &[u32]) -> u32 {\n        let mut sum = 0;\n        for row in rows {\n            if *row > 4 {\n                sum += row * 3;\n            } else {\n                sum += 2;\n            }\n        }\n        if sum > 200 {\n            return 200;\n        }\n        sum\n    }\n}\n";
const MODULES: &str = "mod inner {\n    pub fn a(items: &[u32]) -> u32 {\n        let mut total = 0;\n        for item in items {\n            if *item > 3 {\n                total += item * 2;\n            } else {\n                total += 1;\n            }\n        }\n        if total > 100 {\n            return 100;\n        }\n        total\n    }\n}\nmod other {\n    pub fn b(rows: &[u32]) -> u32 {\n        let mut sum = 0;\n        for row in rows {\n            if *row > 4 {\n                sum += row * 3;\n            } else {\n                sum += 2;\n            }\n        }\n        if sum > 100 {\n            return 100;\n        }\n        sum\n    }\n}\n";
const BIG: &str = "fn p1(items: &[u32]) -> u32 {\n    let mut total = 0;\n    let mut at = 0;\n    while at < items.len() {\n        if items[at] > 3 { total += 2; } else { total += 1; }\n        at += 1;\n    }\n    total\n}\n";
const BIG_AGAIN: &str = "fn p2(rows: &[u32]) -> u32 {\n    let mut sum = 0;\n    let mut to = 0;\n    while to < rows.len() {\n        if rows[to] > 4 { sum += 3; } else { sum += 2; }\n        to += 1;\n    }\n    sum\n}\n";
const SMALL: &str = "fn s1(items: &[u32]) -> u32 {\n    let mut total = 0; for item in items { if *item > 3 { total += item * 2; } else { total += 1; } } if total > 100 { return 100; } total }\n";
const SMALL_AGAIN: &str = "fn s2(rows: &[u32]) -> u32 {\n    let mut sum = 0; for row in rows { if *row > 4 { sum += row * 3; } else { sum += 2; } } if sum > 200 { return 200; } sum }\n";
const SAME_FILE: &str = "fn p1(items: &[u32]) -> u32 {\n    let mut total = 0;\n    let mut at = 0;\n    while at < items.len() {\n        if items[at] > 3 { total += 2; } else { total += 1; }\n        at += 1;\n    }\n    total\n}\nfn p2(rows: &[u32]) -> u32 {\n    let mut sum = 0;\n    let mut to = 0;\n    while to < rows.len() {\n        if rows[to] > 4 { sum += 3; } else { sum += 2; }\n        to += 1;\n    }\n    sum\n}\nfn s1(items: &[u32]) -> u32 {\n    let mut total = 0; for item in items { if *item > 3 { total += item * 2; } else { total += 1; } } if total > 100 { return 100; } total }\nfn s2(rows: &[u32]) -> u32 {\n    let mut sum = 0; for row in rows { if *row > 4 { sum += row * 3; } else { sum += 2; } } if sum > 200 { return 200; } sum }\n";

const CASES: [Case; 17] = [
    Case::new(
        Title::new("renamed with other literals"),
        &[Source::new(Located::new("a.rs"), Text::new(BODY)), Source::new(Located::new("b.rs"), Text::new(RENAMED))],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(
        Title::new("within one file"),
        &[Source::new(Located::new("a.rs"), Text::new(THREE))],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(
        Title::new("other callee"),
        &[Source::new(Located::new("a.rs"), Text::new(BODY)), Source::new(Located::new("b.rs"), Text::new(OTHER_CALL))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("below the floor"),
        &[Source::new(Located::new("a.rs"), Text::new(SHORT_A)), Source::new(Located::new("b.rs"), Text::new(SHORT_B))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("test module"),
        &[Source::new(Located::new("a.rs"), Text::new(TESTS_A)), Source::new(Located::new("b.rs"), Text::new(RENAMED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("const fns"),
        &[Source::new(Located::new("a.rs"), Text::new(CONST_A)), Source::new(Located::new("b.rs"), Text::new(CONST_B))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("across crates"),
        &[
            Source::new(Located::new("crates/x/src/a.rs"), Text::new(BODY)),
            Source::new(Located::new("crates/y/src/b.rs"), Text::new(RENAMED)),
        ],
        Flagged::new(&["crates/x/src/a.rs:1"]),
    ),
    Case::new(
        Title::new("inner match belongs to the outer body"),
        &[
            Source::new(Located::new("a.rs"), Text::new(NESTED_A)),
            Source::new(Located::new("b.rs"), Text::new(NESTED_B)),
        ],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(
        Title::new("target role files"),
        &[
            Source::new(Located::new("tests/a.rs"), Text::new(BODY)),
            Source::new(Located::new("tests/b.rs"), Text::new(RENAMED)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a repeated match"),
        &[Source::new(Located::new("a.rs"), Text::new(MATCH_A)), Source::new(Located::new("b.rs"), Text::new(MATCH_B))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("a repeated closure"),
        &[
            Source::new(Located::new("a.rs"), Text::new(CLOSURE_A)),
            Source::new(Located::new("b.rs"), Text::new(CLOSURE_B)),
        ],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("a repeated method chain"),
        &[Source::new(Located::new("a.rs"), Text::new(CHAIN_A)), Source::new(Located::new("b.rs"), Text::new(CHAIN_B))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("provided trait methods"),
        &[Source::new(Located::new("a.rs"), Text::new(TRAITS))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("inline modules"),
        &[Source::new(Located::new("a.rs"), Text::new(MODULES))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("two families in one file"),
        &[Source::new(Located::new("a.rs"), Text::new(SAME_FILE))],
        Flagged::new(&["a.rs:1", "a.rs:19"]),
    ),
    Case::new(
        Title::new("a smaller family in other files than a bigger one"),
        &[
            Source::new(Located::new("a.rs"), Text::new(BIG)),
            Source::new(Located::new("b.rs"), Text::new(BIG_AGAIN)),
            Source::new(Located::new("c.rs"), Text::new(SMALL)),
            Source::new(Located::new("d.rs"), Text::new(SMALL_AGAIN)),
        ],
        Flagged::new(&["a.rs:1", "c.rs:1"]),
    ),
    Case::new(
        Title::new("a bigger family sorts before a smaller one"),
        &[
            Source::new(Located::new("c.rs"), Text::new(SMALL)),
            Source::new(Located::new("d.rs"), Text::new(SMALL_AGAIN)),
            Source::new(Located::new("a.rs"), Text::new(BIG)),
            Source::new(Located::new("b.rs"), Text::new(BIG_AGAIN)),
        ],
        Flagged::new(&["a.rs:1", "c.rs:1"]),
    ),
];

#[test]
fn bodies_repeated_with_names_and_literals_erased_are_listed_once_per_family() {
    assert_eq!(Sources::mismatches::<RepeatedBodies>(&CASES), Lines::default());
}

#[test]
fn a_finding_lists_its_size_its_copies_and_the_fixes_in_order() {
    let text = Sources::report::<RepeatedBodies>(&[Source::new(Located::new("a.rs"), Text::new(THREE))]).to_string();
    assert!(text.contains("46 tokens in 3 copies, also at a.rs:15, a.rs:29; "), "{text}");
    assert!(text.contains("a standard trait such as From, TryFrom, FromStr or Display, else a generic over an existing trait, else a derive"), "{text}");
    assert!(!text.contains("helper"));
}

#[test]
fn a_nested_family_is_not_listed_beside_its_outer_family() {
    let report = Sources::report::<RepeatedBodies>(&[
        Source::new(Located::new("a.rs"), Text::new(NESTED_A)),
        Source::new(Located::new("b.rs"), Text::new(NESTED_B)),
    ]);
    assert_eq!(report.violations().len(), 1);
}
