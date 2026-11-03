use lint_clones::Clones;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

const CLONED: &str = "fn f(text: &String) -> String {\n    text.clone()\n}\n";
const INLINE_BOUNDARY: &str = "mod boundary {\n    fn f(text: &String) -> String {\n        text.clone()\n    }\n}\n";
const TESTED: &str = "#[cfg(test)]\nmod tests {\n    fn f(text: &String) -> String {\n        text.clone()\n    }\n}\n";
const ARGUED: &str = "fn f(pool: &Pool) -> Pool {\n    pool.clone(1)\n}\n";
const CLONED_ITEMS: &str = "fn f(texts: &[String]) -> Vec<String> {\n    texts.iter().cloned().collect()\n}\n";

const CASES: [Case; 7] = [
    Case::new(
        Title::new("a clone in shipped code"),
        &[Source::new(Located::new("src/a.rs"), Text::new(CLONED))],
        Flagged::new(&["src/a.rs:2"]),
    ),
    Case::new(
        Title::new("a boundary module file"),
        &[Source::new(Located::new("src/boundary.rs"), Text::new(CLONED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("an inline boundary module"),
        &[Source::new(Located::new("src/a.rs"), Text::new(INLINE_BOUNDARY))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("test code"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(CLONED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a test module"),
        &[Source::new(Located::new("src/a.rs"), Text::new(TESTED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a method named clone that takes arguments"),
        &[Source::new(Located::new("src/a.rs"), Text::new(ARGUED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("cloned items are another call"),
        &[Source::new(Located::new("src/a.rs"), Text::new(CLONED_ITEMS))],
        Flagged::new(&[]),
    ),
];

#[test]
fn each_clone_in_the_core_is_reported() {
    assert_eq!(Sources::mismatches::<Clones>(&CASES), Lines::default());
}
