use lint_file::Filed;
use lint_no_early_exits::NoEarlyExits;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

const QUESTION_MARK: &str = "fn f(text: &str) -> Result<u8, E> {\n    let parsed = text.parse()?;\n    Ok(parsed)\n}\n";
const TAIL: &str =
    "fn f(on: Switch) -> u8 {\n    match on {\n        Switch::On => 1,\n        Switch::Off => 0,\n    }\n}\n";
const RETURN: &str = "fn f(on: Switch) -> u8 {\n    if on.lit() {\n        return 1;\n    }\n    0\n}\n";
const CLOSURE: &str = "fn f(items: &[u8]) {\n    items.iter().for_each(|_| {\n        return;\n    });\n}\n";
const CONST_FN: &str = "const fn f() {\n    loop {\n        break;\n    }\n}\n";
const LABELED: &str = "fn f() -> u8 {\n    'found: {\n        break 'found 1;\n    }\n}\n";
const TEST_MODULE: &str =
    "#[cfg(test)]\nmod tests {\n    fn f() {\n        while go() {\n            continue;\n        }\n    }\n}\n";
const INTEGRATION: &str = "#[test]\nfn f() {\n    return;\n}\n";
const TRAIT_IMPL: &str =
    "impl Iterator for A {\n    fn next(&mut self) -> Option<u8> {\n        return None;\n    }\n}\n";
const MACRO: &str = "fn f() -> Vec<u8> {\n    vec![{\n        return Vec::new();\n    }]\n}\n";

const CASES: [Case; 10] = [
    Case::new(
        Title::new("question mark"),
        &[Source::new(Located::new("a.rs"), Text::new(QUESTION_MARK))],
        Flagged::new(&[]),
    ),
    Case::new(Title::new("tail value"), &[Source::new(Located::new("a.rs"), Text::new(TAIL))], Flagged::new(&[])),
    Case::new(Title::new("return"), &[Source::new(Located::new("a.rs"), Text::new(RETURN))], Flagged::new(&["a.rs:3"])),
    Case::new(
        Title::new("closure"),
        &[Source::new(Located::new("a.rs"), Text::new(CLOSURE))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("const fn"),
        &[Source::new(Located::new("a.rs"), Text::new(CONST_FN))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("labeled block"),
        &[Source::new(Located::new("a.rs"), Text::new(LABELED))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("test module"),
        &[Source::new(Located::new("a.rs"), Text::new(TEST_MODULE))],
        Flagged::new(&["a.rs:5"]),
    ),
    Case::new(
        Title::new("integration test"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(INTEGRATION))],
        Flagged::new(&["tests/a.rs:3"]),
    ),
    Case::new(
        Title::new("trait impl"),
        &[Source::new(Located::new("a.rs"), Text::new(TRAIT_IMPL))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("inside a macro"),
        &[Source::new(Located::new("a.rs"), Text::new(MACRO))],
        Flagged::new(&["a.rs:3"]),
    ),
];

#[test]
fn return_break_and_continue_are_flagged_in_every_body_and_the_question_mark_is_not() {
    assert_eq!(Sources::mismatches::<Filed<NoEarlyExits>>(&CASES), Lines::default());
}
