use lint_fixtures::{Located, Source, Sources, Text};
use lint_test_strength::TestStrength;
use lint_transition_graph::{ChangeLint, Extraction, TypeSystem};

struct Weakened;

impl Weakened {
    fn lost(before: &[Source], after: &[Source]) -> usize {
        let (before, after) = (Sources::parsed(before), Sources::parsed(after));
        let (before_extraction, after_extraction) =
            (Extraction::from(before.as_slice()), Extraction::from(after.as_slice()));
        let report = TestStrength::report_change(
            &TypeSystem::built_from(&before, &before_extraction),
            &TypeSystem::built_from(&after, &after_extraction),
        );
        report.to_string().matches("lost a check").count()
    }

    fn between(before: &[Source], after: &[Source]) -> bool {
        Self::lost(before, after) > 0
    }

    fn in_one_module(pair: &[&str]) -> bool {
        Some(pair)
            .filter(|pair| pair.len() == 2)
            .and_then(|pair| pair.first().zip(pair.last()))
            .is_some_and(|(before, after)| Self::between(&[Self::module(before)], &[Self::module(after)]))
    }

    fn module(body: &str) -> Source {
        Self::placed("crates/x/src/lib.rs", body)
    }

    fn placed(path: &'static str, body: &str) -> Source {
        Self::at(path, &format!("#[cfg(test)]\nmod tests {{\n{body}\n}}\n"))
    }

    fn at(path: &'static str, text: &str) -> Source {
        Source::new(Located::new(path), Text::new(Box::leak(text.to_owned().into_boxed_str())))
    }
}

const EQUALS: &str = "#[test]\nfn adds() {\n    assert_eq!(f(1), 2);\n}\n";
const BOUNDED: &str = "#[test]\nfn adds() {\n    assert!(f(1) > 0);\n}\n";
const SPELLED_OUT: &str = "#[test]\nfn adds() {\n    assert!(f(1) == 2);\n}\n";
const TWO_ROWS: &str = "laws::table!(adds, f, [1 => 2, 2 => 3]);\n";
const ONE_ROW: &str = "laws::table!(adds, f, [1 => 2]);\n";
const THREE_ROWS: &str = "laws::table!(adds, f, [1 => 2, 2 => 3, 3 => 4]);\n";
const LOOPED: &str = "const ROWS: [(u8, u8); 3] = [(1, 2), (2, 3), (3, 4)];\n#[test]\nfn adds() {\n    for (input, want) in ROWS {\n        assert_eq!(f(input), want);\n    }\n}\n";
const LOOPED_SHORT: &str = "const ROWS: [(u8, u8); 2] = [(1, 2), (2, 3)];\n#[test]\nfn adds() {\n    for (input, want) in ROWS {\n        assert_eq!(f(input), want);\n    }\n}\n";
const ENUMERATED: &str = "#[test]\nfn every_mark_shows() {\n    for mark in Mark::ALL {\n        assert_eq!(mark.show(), mark.name());\n    }\n}\n";
const LISTED: &str = "#[test]\nfn every_mark_shows() {\n    for mark in [Mark::A, Mark::B] {\n        assert_eq!(mark.show(), mark.name());\n    }\n}\n";
const INLINE: &str = "#[test]\nfn adds() {\n    assert_eq!(f(1), 2);\n}\n";
const HELPED: &str = "fn expect(value: u8, want: u8) {\n    assert_eq!(f(value), want);\n}\n#[test]\nfn adds() {\n    expect(1, 2);\n}\n";
const RENAMED: &str = "#[test]\nfn sums() {\n    assert_eq!(f(1), 2);\n}\n";
const SPLIT: &str =
    "#[test]\nfn adds_one() {\n    assert_eq!(f(1), 2);\n}\n#[test]\nfn adds_two() {\n    assert_eq!(f(2), 3);\n}\n";
const JOINED: &str = "#[test]\nfn adds() {\n    assert_eq!(f(1), 2);\n    assert_eq!(f(2), 3);\n}\n";
const EMPTY: &str = "";
const ASSOCIATIVE: &str = "laws::associative!(joins, any::<A>(), A::join);\n";
const COMMUTATIVE: &str = "laws::commutative!(joins, any::<A>(), A::join);\n";
const ARRAY: &str = "#[test]\nfn lists() {\n    assert_eq!(all(), [1, 2, 3]);\n}\n";
const ARRAY_SHORT: &str = "#[test]\nfn lists() {\n    assert_eq!(all(), [1, 2]);\n}\n";
const DIFFERS: &str = "#[test]\nfn differs() {\n    assert_ne!(f(1), f(2));\n}\n";
const BARE: &str = "#[test]\nfn differs() {\n    assert!(f(1));\n}\n";
const TWICE: &str = "#[test]\nfn twice() {\n    assert!(f(1) > 0);\n    assert!(f(1) > 0);\n}\n";
const ONCE: &str = "#[test]\nfn twice() {\n    assert!(f(1) > 0);\n}\n";
const OK_PINNED: &str = "#[test]\nfn parses() {\n    assert_eq!(parse(1).is_ok(), true);\n}\n";
const OK_BARE: &str = "#[test]\nfn parses() {\n    assert!(parse(1).is_ok());\n}\n";
const POSITIVE_THEN_VALUE: &str = "#[test]\nfn mixed() {\n    assert!(f(9) > 0);\n    assert_eq!(f(1), 1);\n}\n";
const VALUE_THEN_POSITIVE: &str = "#[test]\nfn mixed() {\n    assert_eq!(f(1), 1);\n    assert!(f(9) > 0);\n}\n";

laws::table!(a_change_is_flagged_exactly_when_a_test_loses_strength, Weakened::in_one_module, [
    [EQUALS, EQUALS] => false,
    [EQUALS, BOUNDED] => true,
    [EQUALS, SPELLED_OUT] => false,
    [SPELLED_OUT, EQUALS] => false,
    [TWO_ROWS, ONE_ROW] => true,
    [ONE_ROW, TWO_ROWS] => false,
    [TWO_ROWS, THREE_ROWS] => false,
    [LOOPED, LOOPED_SHORT] => true,
    [ENUMERATED, LISTED] => true,
    [INLINE, HELPED] => false,
    [EQUALS, RENAMED] => false,
    [EQUALS, EMPTY] => true,
    [JOINED, SPLIT] => false,
    [ASSOCIATIVE, COMMUTATIVE] => true,
    [ARRAY, ARRAY_SHORT] => true,
    [DIFFERS, BARE] => true,
    [TWICE, ONCE] => true,
    [OK_PINNED, OK_BARE] => false,
    [POSITIVE_THEN_VALUE, VALUE_THEN_POSITIVE] => false,
]);

#[test]
fn a_test_moved_to_another_package_is_flagged() {
    assert!(Weakened::between(
        &[Weakened::placed("crates/x/src/lib.rs", EQUALS)],
        &[Weakened::placed("crates/y/src/lib.rs", EQUALS)],
    ));
}

#[test]
fn a_test_moved_to_another_file_of_its_package_is_not_flagged() {
    assert_eq!(
        Weakened::lost(
            &[Weakened::placed("crates/x/src/a.rs", EQUALS)],
            &[Weakened::placed("crates/x/src/b.rs", EQUALS)],
        ),
        0
    );
}
