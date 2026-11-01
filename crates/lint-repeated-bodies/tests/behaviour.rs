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

const CASES: [Case; 9] = [
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
